use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let Arguments {
        file_name,
        delimeter,
        fields_needed,
    } = Arguments::new()?;
    let reader: Box<dyn BufRead> = match file_name {
        Some(file_name) => {
            let f = File::open(file_name)?;
            Box::new(BufReader::new(f))
        }
        None => {
            let stdin = std::io::stdin();
            let reader = stdin.lock();
            Box::new(reader)
        }
    };
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

    for line in reader.lines() {
        let line = line?;
        for field_needed in &fields_needed {
            let word_needed = line
                .split(&delimeter)
                .nth(*field_needed)
                .unwrap_or_default();
            let word_needed = format!("{word_needed}{delimeter}");
            write_to_output(word_needed.as_bytes())?;
        }
        write_to_output(b"\n")?;
    }

    Ok(())
}

struct Arguments {
    file_name: Option<String>,
    delimeter: String,
    fields_needed: Vec<usize>,
}

impl Arguments {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut fields_needed = vec![];
        let mut delimeter = None;
        let mut file_name = None;
        const SKIP_CHALLENGE_PATH: usize = 1;
        let args = env::args().skip(SKIP_CHALLENGE_PATH);
        const FIELD_COMMAND: &str = "-f";
        const DELIMETER_COMMAND: &str = "-d";
        for arg in args {
            if arg.starts_with(FIELD_COMMAND) {
                let arg = arg.replace(FIELD_COMMAND, "");
                fields_needed = Self::get_fields_needed(&arg)?;
            } else if arg.starts_with(DELIMETER_COMMAND) {
                delimeter = Some(arg.replace(DELIMETER_COMMAND, ""));
            } else if !arg.trim().is_empty() {
                file_name = Some(arg);
            }
        }
        if fields_needed.is_empty() {
            return Err("No fields were provided".into());
        }
        let delimeter = delimeter.unwrap_or_else(|| "\t".to_string());
        Ok(Self {
            file_name,
            delimeter,
            fields_needed,
        })
    }

    fn get_fields_needed(fields_needed: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        let results = if fields_needed.contains(',') {
            fields_needed
                .split(',')
                .map(|field| field.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()?
        } else if fields_needed.contains(' ') {
            fields_needed
                .split(' ')
                .map(|field| field.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![fields_needed.parse::<usize>()?]
        };

        // we are subtracting 1 because the command is 1 based and the field is 0 based
        Ok(results.into_iter().map(|field| field - 1).collect())
    }
}
