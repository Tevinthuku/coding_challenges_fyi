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
    let fields_needed = {
        let fields_needed = command.replace(FILE_COMMAND, "");
        if fields_needed.contains(',') {
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
        }
    };
    const DELIMETER_COMMAND: &str = "-d";
    let maybe_file_name_or_delimeter = args.next().ok_or("Failed to get the next argument")?;
    if maybe_file_name_or_delimeter.starts_with(DELIMETER_COMMAND) {
        let delimeter = maybe_file_name_or_delimeter.replace(DELIMETER_COMMAND, "");
        let file_name = args.next().ok_or("Failed to get the file name")?;
        return process_command_f2(&file_name, &delimeter, fields_needed);
    }
    let default_tab_delimeter = "\t";
    process_command_f2(
        &maybe_file_name_or_delimeter,
        default_tab_delimeter,
        fields_needed,
    )
}

fn process_command_f2(
    file_name: &str,
    delimeter: &str,
    fields_needed: Vec<usize>,
) -> Result<(), Box<dyn Error>> {
    let f = File::open(file_name)?;
    let reader = BufReader::new(f);
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

    let lines = reader.lines();
    for line in lines {
        let line = line?;
        for field_needed in &fields_needed {
            // we are subtracting 1 because the command is 1 based and the field is 0 based
            let field_needed = field_needed - 1;
            let word_needed = line.split(delimeter).nth(field_needed).unwrap_or_default();
            let word_needed = format!("{word_needed}{delimeter}");
            write_to_output(word_needed.as_bytes())?;
        }
        write_to_output(b"\n")?;
    }

    Ok(())
}
