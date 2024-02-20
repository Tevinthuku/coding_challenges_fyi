use anyhow::{anyhow, bail, Context};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Frame {
    SimpleString(String),
    NullBulkString,
    Error(String),
    Integer(i64),
    Boolean(bool),
    Double(f64),
    Array(Vec<Frame>),
    Null,
    BulkString(Bytes),
}

impl Frame {
    pub fn deserialize(content: &BytesMut) -> anyhow::Result<Frame> {
        let mut x = content.iter().peekable();
        deserialize(&mut x)
    }

    pub fn new_error(message: String) -> Frame {
        Frame::Error(message)
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Frame::SimpleString(_) => "SimpleString",
            Frame::BulkString { .. } => "BulkString",
            Frame::NullBulkString => "NullBulkString",
            Frame::Error(_) => "Error",
            Frame::Integer(_) => "Integer",
            Frame::Boolean(_) => "Boolean",
            Frame::Double(_) => "Double",
            Frame::Array(_) => "Array",
            Frame::Null => "Null",
        }
    }

    pub(crate) fn new_bulk_string(content: Bytes) -> Frame {
        Frame::BulkString(content)
    }
}

impl Frame {}

fn deserialize(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<Frame> {
    let ch = match iter.next() {
        Some(ch) => ch,
        None => bail!("Expected a character, but got nothing"),
    };

    let content = match ch {
        b'+' => Ok(Frame::SimpleString(string(iter)?)),
        b'-' => Ok(Frame::Error(string(iter)?)),
        b':' => integer(iter).map(Frame::Integer),
        b'$' if iter.peek() == Some(&&b'-') => {
            null_bulk_string(iter).map(|_| Frame::NullBulkString)
        }
        b'$' => {
            let bytes = bulk_string(iter)?;
            Ok(Frame::BulkString(bytes))
        }
        b'#' => boolean(iter).map(Frame::Boolean),
        b',' => double(iter).map(Frame::Double),
        b'*' => array(iter).map(Frame::Array),
        b'_' => {
            let _ = null(iter);
            Ok(Frame::Null)
        }
        ch => Err(anyhow!("Invalid character: {}", ch)),
    };
    content
}

fn bulk_string(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<Bytes> {
    let length = integer(iter)?;
    if length < 0 {
        bail!("Invalid length for bulk string: {}", length);
    }
    let content = get_bytes(iter)?;
    if content.len() as i64 != length {
        bail!("Invalid content length for bulk string: {:?}", content);
    }
    Ok(content)
}

fn null(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<()> {
    let content = get_bytes(iter)?;
    if !content.is_empty() {
        bail!("Invalid null: {:?}", content);
    }
    Ok(())
}

fn array(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<Vec<Frame>> {
    let length = integer(iter)?;
    if length < 0 {
        bail!("Invalid length for array: {}", length);
    }
    let mut result = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let frame = deserialize(iter);
        result.push(frame?);
    }
    Ok(result)
}

fn double(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<f64> {
    let multiplication_factor = iter
        .next_if(|&&x| x == b'+' || x == b'-')
        .map(|&ch| if ch == b'-' { -1 } else { 1 })
        .unwrap_or(1);

    let number = string(iter)?;

    let number = number
        .parse::<f64>()
        .with_context(|| format!("Invalid double: {}", number))?;

    Ok(number * multiplication_factor as f64)
}

fn boolean(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<bool> {
    let content = get_bytes(iter)?.get_u8();
    match content {
        b't' => Ok(true),
        b'f' => Ok(false),
        _ => bail!("Invalid boolean: {}", content),
    }
}

fn string(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<String> {
    let bytes = get_bytes(iter)?;
    let string = String::from_utf8(bytes.to_vec()).context("Failed to convert to a string")?;

    Ok(string)
}

fn integer(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<i64> {
    let multiplication_factor = iter
        .next_if(|&&x| x == b'+' || x == b'-')
        .map(|&ch| if ch == b'-' { -1 } else { 1 })
        .unwrap_or(1);

    let number = string(iter)?;
    let number = number
        .parse::<i64>()
        .with_context(|| format!("Invalid number: {}", number))?;
    Ok(number * multiplication_factor)
}

fn null_bulk_string(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<()> {
    let length = integer(iter)?;
    if length != -1 {
        bail!("Invalid length for null bulk string: {}", length);
    }
    Ok(())
}

fn get_bytes(iter: &mut Peekable<std::slice::Iter<u8>>) -> anyhow::Result<Bytes> {
    let mut result = BytesMut::new();

    while let Some(ch) = iter.next() {
        match ch {
            b'\r' if matches!(iter.peek(), Some(&b'\n')) => {
                let _dash_n = iter.next();
                return Ok(result.freeze());
            }
            ch => {
                result.put_u8(*ch);
            }
        }
    }

    bail!("Incomplete content")
}

#[cfg(test)]
mod tests {

    use crate::frame::Frame;
    use bytes::BytesMut;
    use rstest::rstest;

    #[rstest]
    #[case("+OK\r\n", Frame::SimpleString("OK".into()))]
    #[case("-Error message\r\n", Frame::Error("Error message".into()))]
    #[case(":1000\r\n", Frame::Integer(1000))]
    #[case(":-1000\r\n", Frame::Integer(-1000))]
    #[case("$6\r\nfoobar\r\n", Frame::BulkString(bytes::Bytes::from("foobar".as_bytes())))]
    #[case("$-1\r\n", Frame::NullBulkString)]
    #[case("#t\r\n", Frame::Boolean(true))]
    #[case("#f\r\n", Frame::Boolean(false))]
    #[case(",3.15\r\n", Frame::Double(3.15_f64))]
    #[case(",-3.15\r\n", Frame::Double(-3.15_f64))]
    #[case(",3\r\n", Frame::Double(3_f64))]
    #[case("*0\r\n", Frame::Array(vec![]))]
    #[case("*2\r\n+Foo\r\n-Bar\r\n", Frame::Array(vec![
        Frame::SimpleString("Foo".into()),
        Frame::Error("Bar".into())
    ]))]
    #[case("_\r\n", Frame::Null)]
    fn test_content(#[case] input: &'static str, #[case] expected: crate::frame::Frame) {
        let input = BytesMut::from(input.as_bytes());
        let result = Frame::deserialize(&input)
            .map_err(|err| {
                println!("Error deserializing: {:?}", input);
                err
            })
            .unwrap();
        assert_eq!(result, expected);
    }
}
