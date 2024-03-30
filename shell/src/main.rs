use std::{
    io::{self, BufRead, Read, Write},
    process::{ChildStdout, Command, Stdio},
};

fn main() -> io::Result<()> {
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
            let first_command_result = execute_command(first_command, None);
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

        let mut output = piped_commands.try_fold(first_command_output, |a, c| {
            let maybe_child = execute_command(c, Some(a));
            match maybe_child {
                Ok(CommandExecution::ChildOutput(out)) => Ok(out),
                Ok(CommandExecution::Exit) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Did not expect exit command in piped commands",
                )),
                Ok(CommandExecution::DirectoryChange) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Did not expect cd command in piped commands",
                )),
                Err(e) => Err(e),
            }
        })?;

        let mut buffer = Vec::new();
        output.read_to_end(&mut buffer)?;
        stdout_handle.write_all(&buffer)?;
    }

    Ok(())
}

enum CommandExecution {
    ChildOutput(ChildStdout),
    Exit,
    DirectoryChange,
}

fn execute_command(
    command: &str,
    input: Option<ChildStdout>,
) -> Result<CommandExecution, io::Error> {
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
    let stdin = input.map_or(Stdio::null(), Stdio::from);
    let child = Command::new(program)
        .args(&args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .spawn()?;
    let output = child.stdout.ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Could not get child process' stdout")
    })?;

    Ok(CommandExecution::ChildOutput(output))
}
