use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

const SKIP_CHALLENGE_PATH: usize = 1;
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);
    let command = args.next().ok_or("Failed to get the command")?;
    const FILE_COMMAND: &str = "-f";
    if !command.starts_with(FILE_COMMAND) {
        return Err("Invalid command".into());
    }
    // we are subtracting 1 because the command is 1 based and the field is 0 based
    let field_needed = command.replace(FILE_COMMAND, "").parse::<usize>()? - 1;
    let file_name = args.next().ok_or("Failed to get the file name")?;
    process_command_f2(&file_name, field_needed)
}

fn process_command_f2(file_name: &str, field_needed: usize) -> Result<(), Box<dyn Error>> {
    let f = File::open(file_name)?;
    let reader = BufReader::new(f);
    let mut stdout = io::stdout().lock();

    let lines = reader.lines();
    for line in lines {
        let line = line?;
        let second_word = line.split('\t').nth(field_needed).unwrap_or_default();
        stdout.write_all(second_word.as_bytes())?;
        stdout.write_all(b"\n")?;
    }
    Ok(())
}
