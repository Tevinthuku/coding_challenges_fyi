use std::{
    io::{self, BufRead, Read, Write},
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

fn main() {
    loop {
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();
        stdout_handle.write_all(b"ccsh>").unwrap();
        stdout_handle.flush().unwrap();
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_line(&mut buffer).unwrap();
        let mut buffer = buffer.split_whitespace();
        let command = buffer.next().unwrap();
        let args = buffer.collect::<Vec<&str>>();
        let output = Command::new(command).args(&args).output().unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
    }
}
