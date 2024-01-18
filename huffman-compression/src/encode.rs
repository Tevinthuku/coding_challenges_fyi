use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fs,
    io::{BufRead, Read, Write},
};

use bit_vec::BitVec;
use itertools::Itertools;

use crate::{code_generation::generate_codes, whitespace::unicode_encoding};

fn write_header<W: Write>(
    writer: &mut W,
    huffman_codes: &HashMap<char, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    // Write a marker to indicate the start of the header
    writeln!(writer, "HEADER_START")?;

    // Write the character frequency table
    for (character, code) in huffman_codes.iter() {
        writeln!(
            writer,
            "{}:{}",
            unicode_encoding(*character),
            serde_json::to_string(code)?
        )?;
    }

    // Write a marker to indicate the end of the header
    writeln!(writer, "HEADER_END")?;

    Ok(())
}

fn encode<W: Write>(
    input_file: impl AsRef<str>,
    writer: &mut W,
) -> Result<HashMap<char, Vec<u8>>, Box<dyn Error>> {
    let content = fs::read_to_string(input_file.as_ref())?;
    let huffman_codes = {
        let mapping = content
            .chars()
            .into_grouping_map_by(|&x| x)
            .fold(0, |acc, _key, _value| acc + 1);
        let codes = generate_codes(mapping)?;

        let codes = match codes {
            Some(codes) => codes,
            None => return Ok(Default::default()),
        };

        codes
    };

    write_header(writer, &huffman_codes)?;

    let encoded_content = content
        .chars()
        .flat_map(|ch| huffman_codes[&ch].iter())
        .copied()
        .collect_vec();

    writer.write_all(&encoded_content)?;

    Ok(huffman_codes)
}

fn read_header<R: BufRead>(reader: &mut R) -> std::io::Result<HashMap<char, Vec<u8>>> {
    let mut header_lines = Vec::new();

    // Read lines until "HEADER_END" is encountered
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.trim() == "HEADER_END" {
            break;
        }

        header_lines.push(line);
    }

    // Process header lines to build the prefix table
    let mut prefix_table: HashMap<char, Vec<u8>> = HashMap::new();
    for line in header_lines {
        let parts: Vec<&str> = line.trim().split(':').collect();
        if parts.len() == 2 {
            let character = parts[0].chars().next().unwrap_or_default();
            let code_as_str = parts[1];
            let bytes: Vec<u8> = serde_json::from_str(code_as_str)?;
            prefix_table.insert(character, bytes);
        }
    }

    Ok(prefix_table)
}

fn huffman_decode_2<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> std::io::Result<()> {
    let huffman_codes = read_header(reader)?;
    let mut byte_buffer = Vec::new();

    loop {
        // Read a byte from the input stream
        let mut buffer = [0; 1];
        if reader.read_exact(&mut buffer).is_err() {
            break; // End of file
        }

        byte_buffer.extend(buffer);

        for (ch, code_bytes) in huffman_codes.iter() {
            if code_bytes == &byte_buffer {
                println!("{ch} char found");
                write!(writer, "{}", ch)?;
                byte_buffer.clear();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, BufWriter},
    };

    use crate::encode::huffman_decode_2;

    use super::{encode, read_header};

    #[test]
    fn test_encoding() {
        let file = "output-4.txt";
        let output_file = File::create(file).unwrap();
        let mut writer = BufWriter::new(output_file);
        let codes = encode("lorem.txt", &mut writer).unwrap();
        drop(writer);
        println!("{codes:?}");

        let input_file = File::open(file).unwrap();
        let mut reader = BufReader::new(input_file);
        let output_file_after_decompression = "output-decompression.txt";
        let file = File::create(output_file_after_decompression).unwrap();
        let mut writer = BufWriter::new(file);
        let w = '\u{0009}';
        huffman_decode_2(&mut reader, &mut writer).unwrap();
    }
}
