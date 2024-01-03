use std::{
    env,
    error::Error,
    fs::File,
    io::{BufReader, Read},
};

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);
    let command = args.next().ok_or("Failed to get the command")?;
    let file_name = args.next().ok_or("Failed to get the file name")?;
    if command != "-c" {
        return Err(format!("Unexpected command {command} expected -c").into());
    }

    let file = File::open(&file_name)?;
    let buf_reader = BufReader::new(file);
    let count_of_bytes = buf_reader.bytes().count();

    println!("{count_of_bytes} {file_name}");

    Ok(())
}
