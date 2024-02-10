use std::iter::Peekable;

use anyhow::{anyhow, bail, Context};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq, Clone)]
pub enum Frame {
    SimpleString(String),
    BulkString { content: String, length: usize },
    NullBulkString,
    Error(String),
    Integer(i64),
    Boolean(bool),
    Double(f64),
    Array(Vec<Frame>),
    Null,
}

impl Frame {
    pub fn deserialize(content: String) -> Option<anyhow::Result<Frame>> {
        let mut chars = content.chars().peekable();
        deserialize(&mut chars)
    }
    pub fn serialize(self) -> Bytes {
        let mut buf = BytesMut::new();
        match self {
            Frame::SimpleString(content) => {
                buf.put(&b"+"[..]);
                buf.put(content.as_bytes());
                buf.put(&b"\r\n"[..]);
            }
            Frame::Error(content) => {
                buf.put(&b"-"[..]);
                buf.put(content.as_bytes());
                buf.put(&b"\r\n"[..]);
            }
            Frame::BulkString { content, length } => {
                buf.put(&b"$"[..]);
                buf.put(length.to_string().as_bytes());
                buf.put(&b"\r\n"[..]);
                buf.put(content.as_bytes());
                buf.put(&b"\r\n"[..]);
            }
            _ => unimplemented!(),
        }
        buf.freeze()
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

    pub(crate) fn new_bulk_string(content: String) -> Frame {
        let length = content.len();
        Frame::BulkString { content, length }
    }
}

fn deserialize(iter: &mut Peekable<std::str::Chars>) -> Option<anyhow::Result<Frame>> {
    let ch = match iter.next() {
        Some(ch) => ch,
        None => return None,
    };

    let content = match ch {
        '+' => Ok(Frame::SimpleString(string(iter))),
        '-' => Ok(Frame::Error(string(iter))),
        ':' => integer(iter).map(Frame::Integer),
        '$' if iter.peek() == Some(&'-') => null_bulk_string(iter).map(|_| Frame::NullBulkString),
        '$' => bulk_string(iter).map(|(content, length)| Frame::BulkString { content, length }),
        '#' => boolean(iter).map(Frame::Boolean),
        ',' => double(iter).map(Frame::Double),
        '*' => array(iter).map(Frame::Array),
        '_' => {
            let _ = null(iter);
            Ok(Frame::Null)
        }
        ch => Err(anyhow!("Invalid character: {}", ch)),
    };
    Some(content)
}

fn boolean(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<bool> {
    let content = string(iter);
    match content.as_str() {
        "t" => Ok(true),
        "f" => Ok(false),
        _ => bail!("Invalid boolean: {}", content),
    }
}

fn array(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<Vec<Frame>> {
    let length = integer(iter)?;
    if length < 0 {
        bail!("Invalid length for array: {}", length);
    }
    let mut result = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let frame = deserialize(iter).transpose()?;
        if let Some(frame) = frame {
            result.push(frame);
        } else {
            let current_length = result.len();
            bail!("Expected {length} elements in array, but got {current_length}")
        }
    }
    Ok(result)
}

fn bulk_string(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<(String, usize)> {
    let length = integer(iter)?;
    if length < 0 {
        bail!("Invalid length for bulk string: {}", length);
    }
    let content = string(iter);
    Ok((content, length as usize))
}

fn null_bulk_string(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<()> {
    let length = integer(iter)?;
    if length != -1 {
        bail!("Invalid length for null bulk string: {}", length);
    }
    let content = string(iter);
    if !content.is_empty() {
        bail!("Invalid content for null bulk string: {}", content);
    }
    Ok(())
}

fn integer(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<i64> {
    let multiplication_factor = iter
        .next_if(|&x| x == '+' || x == '-')
        .map(|ch| if ch == '-' { -1 } else { 1 })
        .unwrap_or(1);

    let number = string(iter);
    let number = number
        .parse::<i64>()
        .with_context(|| format!("Invalid number: {}", number))?;
    Ok(number * multiplication_factor)
}

fn double(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<f64> {
    let multiplication_factor = iter
        .next_if(|&x| x == '+' || x == '-')
        .map(|ch| if ch == '-' { -1 } else { 1 })
        .unwrap_or(1);

    let number = string(iter);

    let number = number
        .parse::<f64>()
        .with_context(|| format!("Invalid double: {}", number))?;

    Ok(number * multiplication_factor as f64)
}

fn null(iter: &mut Peekable<std::str::Chars>) -> anyhow::Result<()> {
    let content = string(iter);
    if !content.is_empty() {
        bail!("Invalid null: {}", content);
    }
    Ok(())
}

fn string(iter: &mut Peekable<std::str::Chars>) -> String {
    let mut result = String::new();

    while let Some(ch) = iter.next() {
        match ch {
            '\r' if matches!(iter.peek(), Some('\n')) => {
                let _dash_n = iter.next();
                break;
            }
            ch => {
                result.push(ch);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {

    use super::Frame;
    use rstest::rstest;

    #[rstest]
    #[case("+OK\r\n", Frame::SimpleString("OK".into()))]
    #[case("-Error message\r\n", Frame::Error("Error message".into()))]
    #[case(":1000\r\n", Frame::Integer(1000))]
    #[case(":-1000\r\n", Frame::Integer(-1000))]
    #[case("$6\r\nfoobar\r\n", Frame::BulkString {
        content: "foobar".to_string(),
        length: 6
    })]
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
    fn test_content(#[case] input: &'static str, #[case] expected: super::Frame) {
        let result = Frame::deserialize(input.to_owned()).unwrap().unwrap();
        assert_eq!(result, expected);
    }
}
