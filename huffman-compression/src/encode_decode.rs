use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
};

use itertools::Itertools;

use crate::prefix_code_table::generate_codes;

const HEADER_END_INDICATION: &str = "HEADER_END";

fn encode<W: Write>(input_file: impl AsRef<str>, writer: &mut W) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(input_file.as_ref())?;
    let huffman_codes = {
        let codes = generate_codes(&content)?;

        match codes {
            Some(codes) => codes,
            None => return Ok(()),
        }
    };
    write_header(writer, &huffman_codes)?;

    let encoded_content = content
        .chars()
        .flat_map(|ch| huffman_codes[&ch].iter())
        .copied()
        .collect_vec();

    writer.write_all(&encoded_content)?;

    Ok(())
}

fn write_header<W: Write>(
    writer: &mut W,
    huffman_codes: &HashMap<char, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    for (character, code) in huffman_codes.iter() {
        writeln!(
            writer,
            "{}:{}",
            unicode_encoding(*character),
            serde_json::to_string(code)?
        )?;
    }

    // we don't want to save the actual white space
    // but their unicode encoding
    fn unicode_encoding(input: char) -> String {
        match input {
            '\t' => "\\u{0009}".to_owned(),
            '\n' => "\\u{000A}".to_owned(),
            '\r' => "\\u{000D}".to_owned(),
            ' ' => "\\u{0020}".to_owned(),
            c => c.to_string(),
        }
    }

    writeln!(writer, "{}", HEADER_END_INDICATION)?;

    Ok(())
}

fn huffman_decode<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> std::io::Result<()> {
    let huffman_codes = read_header(reader)?;

    let huffman_codes = huffman_codes
        .into_iter()
        .map(|(ch, codes)| (codes, ch))
        .collect::<HashMap<_, _>>();

    let mut bit_buffer = Vec::new();

    loop {
        let mut buffer = [0; 8];
        if reader.read_exact(&mut buffer).is_err() {
            break; // End of file
        }

        for bit in buffer {
            bit_buffer.push(bit);
            if let Some(ch) = huffman_codes.get(&bit_buffer) {
                write!(writer, "{}", ch)?;
                bit_buffer.clear();
            }
        }
    }
    Ok(())
}

fn read_header<R: BufRead>(reader: &mut R) -> std::io::Result<HashMap<char, Vec<u8>>> {
    let mut header_lines = Vec::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|err| {
            std::io::Error::new(err.kind(), format!("failed to read header line: {}", err))
        })?;

        if line.trim() == HEADER_END_INDICATION {
            break;
        }

        header_lines.push(line);
    }

    let mut huffman_codes = HashMap::with_capacity(header_lines.len());
    for (line_number, line) in header_lines.iter().enumerate() {
        let parts: Vec<&str> = if line.contains("::") {
            // since we are splitting by ":", the colon character will be treated as the separator;
            // this is why we handle it separately in this if block
            let split: Vec<&str> = line.trim().split("::").collect();
            let codes = split.last().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to get the bytes for line_number = {line_number}",),
                )
            })?;

            vec![":", codes]
        } else {
            line.trim().split(':').collect()
        };
        if parts.len() < 2 {
            continue;
        }

        let character = unicode_decoding(parts[0])
            .chars()
            .next()
            .unwrap_or_default();
        let code_as_str = parts[1];

        let bytes = serde_json::from_str::<Vec<u8>>(code_as_str)?;

        huffman_codes.insert(character, bytes);
    }

    fn unicode_decoding(input: &str) -> &str {
        match input {
            "\\u{0009}" => "\t",
            "\\u{000A}" => "\n",
            "\\u{000D}" => "\r",
            "\\u{0020}" => " ",
            c => c,
        }
    }

    Ok(huffman_codes)
}

pub fn encode_and_decode(
    input_file: impl AsRef<str>,
    output_file: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let input_file = input_file.as_ref();

    let (mut encoding_writer, encoded_file) = {
        let encoded_file = "encoded.bin";
        let output_file = File::create(encoded_file)?;
        (BufWriter::new(output_file), encoded_file)
    };
    encode(input_file, &mut encoding_writer)?;

    drop(encoding_writer);

    let mut encoded_file_reader = BufReader::new(File::open(encoded_file)?);

    let mut decoded_content_writer = BufWriter::new(File::create(output_file.as_ref())?);
    huffman_decode(&mut encoded_file_reader, &mut decoded_content_writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::encode_and_decode;

    #[test]
    fn test_encoding_decoding() {
        let input_file = "./tests/lorem.txt";
        let output_file = "./tests/lorem-output.txt";

        encode_and_decode(input_file, output_file).unwrap();

        let input_content = include_str!("../tests/lorem.txt");
        let output_content = include_str!("../tests/lorem-output.txt");

        assert_eq!(input_content, output_content);
    }
}
