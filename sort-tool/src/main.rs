use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use itertools::Itertools;

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);
    let file_name = args.next().ok_or("Failed to get file_name")?;
    let reader = BufReader::new(File::open(file_name)?);

    let sorted_content = reader.lines().flatten().sorted();

    let mut stdout = io::stdout().lock();

    let mut write_to_output = |content: &[u8]| -> Result<(), Box<dyn Error>> {
        if let Err(err) = stdout.write_all(content) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            } else {
                return Err(err.into());
            }
        }
        Ok(())
    };

    for content in sorted_content {
        write_to_output(format!("{}\n", content).as_bytes())?;
    }

    Ok(())
}
