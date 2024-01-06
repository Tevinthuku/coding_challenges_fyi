use std::{error::Error, iter::Peekable};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Token {
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    String(String),
    Boolean,
    Null,
    Digit,
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

    fn identifier(&mut self, initial_char: char) -> Result<Token, &'static str> {
        let mut maybe_identifier = initial_char.to_string();
        while let Some(i) = self.input.next_if(|c| c.is_alphanumeric()) {
            maybe_identifier.push(i);
        }
        let identifier = match maybe_identifier.as_str() {
            "true" => Token::Boolean,
            "false" => Token::Boolean,
            "null" => Token::Null,
            _ => return Err("Unknown identifier"),
        };
        Ok(identifier)
    }

    fn digit(&mut self, initial_digit: char) -> Result<Token, Box<dyn Error>> {
        let mut maybe_digit = initial_digit.to_string();

        while let Some(i) = self.input.next_if(|c| c.is_numeric()) {
            maybe_digit.push(i);
        }

        if let Some(i) = self.input.next_if_eq(&'.') {
            maybe_digit.push(i);

            while let Some(i) = self.input.next_if(|c| c.is_numeric()) {
                maybe_digit.push(i);
            }
        }

        maybe_digit
            .parse::<f64>()
            .map(|_| Token::Digit)
            .map_err(|_| "Failed to parse the digit".into())
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
            c if c.is_numeric() => Some(self.digit(c)),
            identifier => Some(self.identifier(identifier).map_err(Into::into)),
        }
    }
}

pub fn tokenize(input: &'static str) -> Result<Vec<Token>, Box<dyn Error>> {
    let tokenizer = Tokenizer::new(input);

    tokenizer.into_iter().collect()
}
