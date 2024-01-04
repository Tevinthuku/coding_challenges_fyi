use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Token {
    LeftBracket,
    RightBracket,
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
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.input.next()?;
        match ch {
            '{' => Some(Token::LeftBracket),
            '}' => Some(Token::RightBracket),
            c if c.is_whitespace() => None,
            _ => unimplemented!(),
        }
    }
}

pub fn tokenize(input: &'static str) -> Vec<Token> {
    let tokenizer = Tokenizer::new(input);

    tokenizer.into_iter().collect()
}
