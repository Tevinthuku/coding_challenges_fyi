pub mod sort;

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

    let content = if args.unique {
        lines.unique().collect_vec()
    } else {
        lines.collect_vec()
    };

    let sorted_content = sort::sort(content, &args.sort_algorithm);

    for content in sorted_content {
        write_to_output(format!("{}\n", content).as_bytes())?;
    }

    Ok(())
}

struct Arguments {
    file_name: String,
    unique: bool,
    sort_algorithm: String,
}

impl Arguments {
    fn new() -> Result<Self, &'static str> {
        let args = env::args().skip(SKIP_CHALLENGE_PATH);
        let mut file_name = None;
        let mut unique = false;
        let mut sort_algorithm = None;
        for arg in args {
            if arg.ends_with(".txt") {
                file_name = Some(arg)
            } else if &arg == "-u" {
                unique = true;
            } else if arg.starts_with("-sort=") {
                sort_algorithm = Some(arg.replace("-sort=", ""));
            } else if arg == "-random-sort" || arg == "-R" {
                sort_algorithm = Some("randomsort".to_string());
            }
        }
        let file_name = file_name.ok_or("Failed to get file_name")?;

        Ok(Arguments {
            file_name,
            unique,
            sort_algorithm: sort_algorithm.unwrap_or_default(),
        })
    }
}
