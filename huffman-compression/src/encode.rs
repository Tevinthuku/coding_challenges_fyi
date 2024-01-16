use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fs,
    io::{BufRead, Read, Write},
};

use bit_vec::BitVec;
use itertools::Itertools;

use crate::code_generation::generate_codes;

fn write_header<W: Write>(
    writer: &mut W,
    huffman_codes: &HashMap<char, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    // Write a marker to indicate the start of the header
    writeln!(writer, "HEADER_START")?;

    // Write the character frequency table
    for (character, code) in huffman_codes.iter() {
        writeln!(writer, "{}:{}", character, code.len())?;
    }

    // Write a marker to indicate the end of the header
    writeln!(writer, "HEADER_END")?;

    Ok(())
}

fn encode<W: Write>(input_file: impl AsRef<str>, writer: &mut W) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(input_file.as_ref())?;
    let huffman_codes = {
        let mapping = content
            .chars()
            .into_grouping_map_by(|&x| x)
            .fold(0, |acc, _key, _value| acc + 1);
        let codes = generate_codes(mapping)?;

        let codes = match codes {
            Some(codes) => codes,
            None => return Ok(()),
        };

        codes
            .into_iter()
            .map(|(ch, bytes)| {
                let bit_vec = BitVec::<u8>::from_iter(bytes.into_iter().map(|b| b == 1)).to_bytes();
                println!("{bit_vec:?}");
                (ch, bit_vec)
            })
            .collect::<HashMap<_, _>>()
    };

    write_header(writer, &huffman_codes)?;

    let encoded_content = content
        .chars()
        .flat_map(|ch| huffman_codes[&ch].iter())
        .copied()
        .collect_vec();
    println!("{encoded_content:?}");
    writer.write_all(&encoded_content)?;

    Ok(())
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
            let code_length: usize = parts[1].parse().unwrap();
            let code_bits = read_bits(reader, code_length)?;
            prefix_table.insert(character, code_bits);
        }
    }

    Ok(prefix_table)
}

fn read_bits<R: Read>(reader: &mut R, num_bits: usize) -> std::io::Result<Vec<u8>> {
    let mut bits = Vec::with_capacity(num_bits);

    for _ in 0..num_bits {
        let mut buffer = [0; 1];
        reader.read_exact(&mut buffer)?;
        bits.push(buffer[0]);
    }
    println!("{bits:?}");
    Ok(bits)
}

fn huffman_decode<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prefix_table: &HashMap<char, Vec<u8>>,
) -> std::io::Result<()> {
    // let mut bit_buffer = VecDeque::new();
    // let mut decoded_chars = String::new();

    loop {
        // Read a byte from the input stream
        let mut buffer = [0; 1];
        if reader.read_exact(&mut buffer).is_err() {
            break; // End of file
        }
        println!("{buffer:?}");

        // Convert the byte into a Vec of bits
        // let byte_bits = (0..8)
        //     .map(|i| (buffer[0] >> i) & 1)
        //     .rev()
        //     .collect::<Vec<u8>>();

        // println!("{byte_bits:?}");
    }

    // Write the decoded content to the output file
    // writer.write_all(decoded_chars.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, BufWriter},
    };

    use super::{encode, huffman_decode, read_header};

    #[test]
    fn test_encoding() {
        let file = "output-4.txt";
        let output_file = File::create(file).unwrap();
        let mut writer = BufWriter::new(output_file);
        encode("lorem.txt", &mut writer).unwrap();
        drop(writer);

        let input_file = File::open(file).unwrap();
        let mut reader = BufReader::new(input_file);

        let prefix_table = read_header(&mut reader).unwrap();
        println!("prefix_table = {prefix_table:?}");
        let output_file = File::create("decoded-lorem.txt").unwrap();
        let mut writer = BufWriter::new(output_file);
        huffman_decode(&mut reader, &mut writer, &prefix_table).unwrap();
    }
}
