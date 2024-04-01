use std::{
    io::{self, BufRead, Write},
    process::Stdio,
};

use futures::{stream, StreamExt, TryStreamExt};
use tokio::{
    io::AsyncReadExt,
    process::{ChildStdout, Command},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();
    let stdin = std::io::stdin();
    let mut stdin_handle = stdin.lock();
    loop {
        stdout_handle.write_all(b"ccsh> ")?;
        stdout_handle.flush()?;
        let mut buffer = String::new();
        stdin_handle.read_line(&mut buffer)?;
        if buffer.trim().is_empty() {
            continue;
        }
        let mut piped_commands = buffer.split('|');

        let first_command_output = {
            let first_command = piped_commands
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not get command"))?;
            let first_command_result = execute_command(first_command, None).await;
            match first_command_result {
                Ok(CommandExecution::ChildOutput(child)) => child,
                Ok(CommandExecution::Exit) => break,
                Ok(CommandExecution::DirectoryChange) => continue,
                Err(e) => {
                    io::stderr().write_all(format!("{e}\n").as_bytes())?;
                    continue;
                }
            }
        };

        let commands_stream = stream::iter(piped_commands);

        let output = commands_stream
            .map(Ok)
            .try_fold(first_command_output, |a, command| async move {
                let command = execute_command(command, Some(a)).await?;
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

        let mut output = match output {
            Ok(output) => output,
            Err(CommandExecutionError::CtrlC) => {
                continue;
            }
            Err(CommandExecutionError::Io(err)) => return Err(err),
        };

        let mut bytes = Vec::new();
        output.read_to_end(&mut bytes).await?;
        stdout_handle.write_all(&bytes)?;
    }

    Ok(())
}

enum CommandExecution {
    ChildOutput(ChildStdout),
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
    input: Option<ChildStdout>,
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
    let stdin = match input {
        Some(out) => ChildStdout::try_into(out)?,
        None => Stdio::null(),
    };
    let mut child = Command::new(program)
        .args(&args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .spawn()?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            let _ = child.kill().await;
            return Err(CommandExecutionError::CtrlC);
        }
        _ = child.wait() => {}
    }

    let output = child.stdout.take().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Could not get child process' stdout")
    })?;

    Ok(CommandExecution::ChildOutput(output))
}

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
