use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use itertools::Itertools;

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
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

    let args = Arguments::new()?;
    let reader = BufReader::new(File::open(args.file_name)?);

    let lines = reader.lines().flatten();

    let sorted_content = if args.unique {
        lines.unique().sorted()
    } else {
        lines.sorted()
    };

    for content in sorted_content {
        write_to_output(format!("{}\n", content).as_bytes())?;
    }

    Ok(())
}

struct Arguments {
    file_name: String,
    unique: bool,
}

impl Arguments {
    fn new() -> Result<Self, &'static str> {
        let args = env::args().skip(SKIP_CHALLENGE_PATH);
        let mut file_name = None;
        let mut unique = false;
        for arg in args {
            if arg.ends_with(".txt") {
                file_name = Some(arg)
            } else if &arg == "-u" {
                unique = true;
            }
        }
        let file_name = file_name.ok_or("Failed to get file_name")?;

        Ok(Arguments { file_name, unique })
    }
}
