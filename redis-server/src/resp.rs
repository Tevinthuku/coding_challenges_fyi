use std::iter::Peekable;

use anyhow::{bail, Context};

#[derive(Debug, PartialEq, Clone)]
enum Token {
    SimpleString(String),
    BulkString { content: String, length: usize },
    NullBulkString,
    Error(String),
    Integer(i64),
    Boolean(bool),
    Double(f64),
    Array(Vec<Token>),
}

struct Tokenizer(Peekable<std::str::Chars<'static>>);

impl Tokenizer {
    fn new(content: &'static str) -> Self {
        let chars = content.chars().peekable();
        Tokenizer(chars)
    }

    fn string(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.0.next() {
            match ch {
                '\r' if matches!(self.0.peek(), Some('\n')) => {
                    let _dash_n = self.0.next();
                    break;
                }
                ch => {
                    result.push(ch);
                }
            }
        }
        result
    }

    fn integer(&mut self) -> anyhow::Result<i64> {
        let multiplication_factor = self
            .0
            .next_if(|&x| x == '+' || x == '-')
            .map(|ch| if ch == '-' { -1 } else { 1 })
            .unwrap_or(1);

        let number = self.string();
        let number = number
            .parse::<i64>()
            .with_context(|| format!("Invalid number: {}", number))?;
        Ok(number * multiplication_factor)
    }

    fn double(&mut self) -> anyhow::Result<f64> {
        let multiplication_factor = self
            .0
            .next_if(|&x| x == '+' || x == '-')
            .map(|ch| if ch == '-' { -1 } else { 1 })
            .unwrap_or(1);

        let number = self.string();

        let number = number
            .parse::<f64>()
            .with_context(|| format!("Invalid double: {}", number))?;

        Ok(number * multiplication_factor as f64)
    }

    fn bulk_string(&mut self) -> anyhow::Result<(String, usize)> {
        let length = self.integer()?;
        if length < 0 {
            bail!("Invalid length for bulk string: {}", length);
        }
        let content = self.string();
        Ok((content, length as usize))
    }

    fn null_bulk_string(&mut self) -> anyhow::Result<()> {
        let length = self.integer()?;
        if length != -1 {
            bail!("Invalid length for null bulk string: {}", length);
        }
        let content = self.string();
        if !content.is_empty() {
            bail!("Invalid content for null bulk string: {}", content);
        }
        Ok(())
    }

    fn boolean(&mut self) -> anyhow::Result<bool> {
        let content = self.string();
        match content.as_str() {
            "t" => Ok(true),
            "f" => Ok(false),
            _ => bail!("Invalid boolean: {}", content),
        }
    }

    fn array(&mut self) -> anyhow::Result<Vec<Token>> {
        let length = self.integer()?;
        if length < 0 {
            bail!("Invalid length for array: {}", length);
        }
        let mut result = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let token = self.deserialize().transpose()?;
            if let Some(token) = token {
                result.push(token);
            } else {
                let current_length = result.len();
                bail!("Expected {length} elements in array, but got {current_length}")
            }
        }
        Ok(result)
    }

    fn deserialize(&mut self) -> Option<anyhow::Result<Token>> {
        let ch = match self.0.next() {
            Some(ch) => ch,
            None => return None,
        };

        let content = match ch {
            '+' => Ok(Token::SimpleString(self.string())),
            '-' => Ok(Token::Error(self.string())),
            ':' => self.integer().map(Token::Integer),
            '$' if self.0.peek() == Some(&'-') => {
                self.null_bulk_string().map(|_| Token::NullBulkString)
            }
            '$' => self
                .bulk_string()
                .map(|(content, length)| Token::BulkString { content, length }),
            '#' => self.boolean().map(Token::Boolean),
            ',' => self.double().map(Token::Double),
            '*' => self.array().map(Token::Array),
            _ => todo!(),
        };
        Some(content)
    }
}

#[cfg(test)]
mod tests {
    use super::Token;
    use super::Tokenizer;
    use rstest::rstest;

    #[rstest]
    #[case("+OK\r\n", Token::SimpleString("OK".to_string()))]
    #[case("-Error message\r\n", Token::Error("Error message".to_string()))]
    #[case(":1000\r\n", Token::Integer(1000))]
    #[case(":-1000\r\n", Token::Integer(-1000))]
    #[case("$6\r\nfoobar\r\n", Token::BulkString {
        content: "foobar".to_string(),
        length: 6
    })]
    #[case("$-1\r\n", Token::NullBulkString)]
    #[case("#t\r\n", Token::Boolean(true))]
    #[case("#f\r\n", Token::Boolean(false))]
    #[case(",3.15\r\n", Token::Double(3.15_f64))]
    #[case(",-3.15\r\n", Token::Double(-3.15_f64))]
    #[case(",3\r\n", Token::Double(3_f64))]
    #[case("*0\r\n", Token::Array(vec![]))]
    #[case("*2\r\n+Foo\r\n-Bar\r\n", Token::Array(vec![
        Token::SimpleString("Foo".to_string()),
        Token::Error("Bar".to_string())
    ]))]
    fn test_content(#[case] input: &'static str, #[case] expected: super::Token) {
        let mut tokenizer = Tokenizer::new(input);
        let result = tokenizer.deserialize().unwrap().unwrap();
        assert_eq!(result, expected);
    }
}
