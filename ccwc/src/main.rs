use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let maybe_command_or_file_name = args.next().ok_or("Failed to get the command")?;

    if maybe_command_or_file_name.starts_with('-') {
        let maybe_file_name = args.next();
        let reader = get_reader(maybe_file_name.clone())?;
        let count = run_command(&maybe_command_or_file_name, reader)?;
        let file_name = maybe_file_name.unwrap_or_default();
        println!("{count} {file_name}");
        return Ok(());
    }

    let file_name = maybe_command_or_file_name;

    let LinesWordsAndCharCount {
        line_count,
        word_count,
        char_count,
    } = line_word_and_char_count(new_buf_reader(&file_name)?)?;

    println!("{char_count} {line_count} {word_count} {file_name}");
    Ok(())
}

fn get_reader(maybe_file_name: Option<String>) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
    let reader: Box<dyn BufRead> = match maybe_file_name {
        Some(file_name) => Box::new(new_buf_reader(&file_name)?),
        None => {
            let stdin = std::io::stdin();
            let reader = stdin.lock();
            Box::new(reader)
        }
    };
    Ok(reader)
}

fn new_buf_reader(file_name: &str) -> Result<BufReader<File>, Box<dyn Error>> {
    let file = File::open(file_name)
        .map_err(|err| format!("{err:?} : file_name provided = {file_name}"))?;
    let buf_reader = BufReader::new(file);

    Ok(buf_reader)
}

fn run_command(command: &str, buf_reader: impl BufRead) -> Result<usize, Box<dyn Error>> {
    let count = match command {
        "-c" => byte_count(buf_reader),
        "-l" => line_count(buf_reader)?,
        "-w" => word_count(buf_reader)?,
        "-m" => char_count(buf_reader)?,
        command => {
            return Err(
                format!("Unexpected command {command} expected either -c | -l | -w | -m").into(),
            );
        }
    };

    Ok(count)
}

fn byte_count(reader: impl BufRead) -> usize {
    reader.bytes().count()
}

fn line_count(reader: impl BufRead) -> Result<usize, Box<dyn Error>> {
    line_word_and_char_count(reader).map(|res| res.line_count)
}

fn word_count(reader: impl BufRead) -> Result<usize, Box<dyn Error>> {
    line_word_and_char_count(reader).map(|res| res.word_count)
}

fn char_count(reader: impl BufRead) -> Result<usize, Box<dyn Error>> {
    line_word_and_char_count(reader).map(|res| res.char_count)
}

struct LinesWordsAndCharCount {
    line_count: usize,
    word_count: usize,
    char_count: usize,
}

fn line_word_and_char_count(
    mut reader: impl BufRead,
) -> Result<LinesWordsAndCharCount, Box<dyn Error>> {
    let mut char_count = 0;
    let mut line_count = 0;
    let mut word_count = 0;

    loop {
        let mut buf = vec![];
        let num_bytes = reader.read_until(b'\n', &mut buf)?;
        if num_bytes == 0 {
            break;
        }
        let line = String::from_utf8(buf)?;

        char_count += line.chars().count();
        line_count += 1;
        word_count += line.split_whitespace().count();
    }

    Ok(LinesWordsAndCharCount {
        line_count,
        char_count,
        word_count,
    })
}
