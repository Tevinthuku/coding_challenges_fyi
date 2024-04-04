use std::{
    io::{self, BufRead, Read, Write},
    process::Stdio,
};

use futures::{stream, StreamExt, TryStreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStdout, Command},
    sync::mpsc::Receiver,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();
    let stdin = std::io::stdin();
    let mut stdin_handle = stdin.lock();
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(2);
    let command_history_handle = tokio::spawn(async move {
        save_command_history(rx).await;
    });
    loop {
        stdout_handle.write_all(b"ccsh> ")?;
        stdout_handle.flush()?;
        let mut command = String::new();
        stdin_handle.read_line(&mut command)?;
        let command = command.trim();

        if command.is_empty() {
            continue;
        }

        let mut piped_commands = command.split('|');

        let first_command_output = {
            let first_command = piped_commands
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not get command"))?;
            let first_command_result = execute_command(first_command, None).await;
            match first_command_result {
                Ok(CommandExecution::ChildOutput(child)) => child,
                Ok(CommandExecution::Exit) => break,
                Ok(CommandExecution::DirectoryChange) => continue,
                Err(CommandExecutionError::CtrlC) => continue,
                Err(e) => {
                    io::stderr().write_all(format!("{e}\n").as_bytes())?;
                    continue;
                }
            }
        };

        if let Err(err) = tx.send(command.to_owned()).await {
            eprintln!("Could not save command to history: {err}");
        }

        let commands_stream = stream::iter(piped_commands);

        let output = commands_stream
            .map(Ok)
            .try_fold(first_command_output, |prev_child_output, command| async {
                let command = execute_command(command, Some(prev_child_output)).await?;
                let name = command.name();
                if let CommandExecution::ChildOutput(out) = command {
                    Ok(out)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Did not expect {name} command in piped commands"),
                    )
                    .into())
                }
            })
            .await;

        let output = match output {
            Ok(output) => output,
            Err(CommandExecutionError::CtrlC) => {
                continue;
            }
            Err(CommandExecutionError::Io(err)) => return Err(err),
        };

        stdout_handle.write_all(&output)?;
    }

    drop(tx);

    command_history_handle.await?;

    Ok(())
}

fn history_file_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|path| path.join(".ccsh_history"))
}

async fn save_command_history(mut rx: Receiver<String>) {
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    while let Some(command) = rx.recv().await {
        let history_file = match history_file_path() {
            Some(path) => path,
            None => {
                eprintln!("Failed to get history file path");
                return;
            }
        };

        let save_result = OpenOptions::new()
            .create(true)
            .append(true)
            .open(history_file)
            .and_then(|mut file| writeln!(file, "{}", command));

        if let Err(e) = save_result {
            eprintln!("Could not save command to history: {e}");
        }
    }
}

#[derive(Debug)]
enum CommandExecution {
    ChildOutput(Vec<u8>),
    Exit,
    DirectoryChange,
}

impl CommandExecution {
    fn name(&self) -> &str {
        match self {
            CommandExecution::ChildOutput(_) => "ChildOutput",
            CommandExecution::Exit => "Exit",
            CommandExecution::DirectoryChange => "DirectoryChange",
        }
    }
}

async fn execute_command(
    command: &str,
    input: Option<Vec<u8>>,
) -> Result<CommandExecution, CommandExecutionError> {
    let mut command = command.split_whitespace();
    let program = command
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not get program name"))?;
    let args = command.collect::<Vec<&str>>();
    if program == "exit" {
        return Ok(CommandExecution::Exit);
    }
    if program == "cd" {
        let path = args.first().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Could not get path to change directory to",
            )
        })?;
        let path = std::path::Path::new(path);
        std::env::set_current_dir(path)?;
        return Ok(CommandExecution::DirectoryChange);
    }

    if program == "history" {
        if let Some(history_file) = history_file_path() {
            let mut file = std::fs::File::open(history_file)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            return Ok(CommandExecution::ChildOutput(buffer));
        }
    }

    let mut command = Command::new(program);

    let stdin = input
        .is_some()
        .then(Stdio::piped)
        .unwrap_or(Stdio::inherit());

    let mut child = command
        .args(&args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(input) = input {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not get child stdin"))?;
        stdin.write_all(&input).await?;
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            let _ = child.kill().await;
            Err(CommandExecutionError::CtrlC)
        }
        output = get_child_output(&mut child) => {
            let output = output?;
            Ok(CommandExecution::ChildOutput(output))
        }
    }
}

async fn get_child_output(child: &mut Child) -> io::Result<Vec<u8>> {
    use futures::try_join;

    async fn read_to_end(mut io: ChildStdout) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        io.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    let stdout = child.stdout.take().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Could not get child stdout for reading",
        )
    })?;
    let (_, buffer) = try_join!(child.wait(), read_to_end(stdout))?;
    Ok(buffer)
}

#[derive(Debug)]
enum CommandExecutionError {
    Io(io::Error),
    CtrlC,
}

impl From<io::Error> for CommandExecutionError {
    fn from(e: io::Error) -> Self {
        CommandExecutionError::Io(e)
    }
}

impl std::fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandExecutionError::Io(e) => write!(f, "{e}"),
            CommandExecutionError::CtrlC => write!(f, "Ctrl-C pressed"),
        }
    }
}
