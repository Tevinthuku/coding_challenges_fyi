use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
};

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let command = args.next().ok_or("Failed to get the command")?;
    let file_name = args.next().ok_or("Failed to get the file name")?;

    let file = File::open(&file_name)?;
    let mut buf_reader = BufReader::new(file);

    match command.as_str() {
        "-c" => {
            let count_of_bytes = buf_reader.bytes().count();
            println!("{count_of_bytes} {file_name}");
        }
        "-l" => {
            let lines = buf_reader.lines().count();
            println!("{lines} {file_name}")
        }
        "-w" => {
            let mut count = 0;
            for line in buf_reader.lines() {
                let line = line?;
                count += line.split_whitespace().count();
            }

            println!("{count} {file_name}")
        }
        "-m" => {
            let mut count = 0;

            loop {
                let mut buf = vec![];
                let num_bytes = buf_reader.read_until(b'\n', &mut buf)?;
                if num_bytes == 0 {
                    break;
                }
                let count_of_chars = String::from_utf8(buf)?.chars().count();
                count += count_of_chars;
            }

            println!("{count} {file_name}")
        }
        command => {
            return Err(format!("Unexpected command {command} expected -c").into());
        }
    }

    Ok(())
}
