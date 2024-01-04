use std::{error::Error, iter::Peekable};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Token {
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    String(String),
}

struct Tokenizer {
    input: Peekable<std::str::Chars<'static>>,
}

impl Tokenizer {
    fn new(input: &'static str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    fn string(&mut self) -> Result<String, Box<dyn Error>> {
        let mut result = String::new();

        loop {
            match self.input.next() {
                Some('"') => {
                    break;
                }
                Some(ch) => result.push(ch),
                None => return Err("Did not find string closing symbol".into()),
            }
        }
        Ok(result)
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Token, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.input.next()?;
        match ch {
            '{' => Some(Ok(Token::LeftBracket)),
            '}' => Some(Ok(Token::RightBracket)),
            '"' => Some(self.string().map(Token::String)),
            ':' => Some(Ok(Token::Colon)),
            ',' => Some(Ok(Token::Comma)),
            c if c.is_whitespace() => self.next(),
            c => Some(Err(format!("Unexpected token {c:?}").into())),
        }
    }
}

pub fn tokenize(input: &'static str) -> Result<Vec<Token>, Box<dyn Error>> {
    let tokenizer = Tokenizer::new(input);

    tokenizer.into_iter().collect()
}
