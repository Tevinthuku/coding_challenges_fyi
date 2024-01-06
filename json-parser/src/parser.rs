use std::{error::Error, iter::Peekable};

use crate::tokenizer::Token;

const EMPTY_JSON_TOKEN_COUNT: usize = 2;

struct Parser {
    count_of_tokens: usize,
    input: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Result<Self, &'static str> {
        if tokens.len() < EMPTY_JSON_TOKEN_COUNT {
            return Err("Invalid json. Not enough tokens provided");
        }
        Ok(Self {
            count_of_tokens: tokens.len(),
            input: tokens.into_iter().peekable(),
        })
    }

    fn next(&mut self) -> Option<Token> {
        self.input.next()
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let _opening_bracket = self
            .input
            .next_if_eq(&Token::LeftBracket)
            .ok_or("Expected {")?;
        if self.count_of_tokens == EMPTY_JSON_TOKEN_COUNT {
            self.input
                .next_if_eq(&Token::RightBracket)
                .ok_or("Expected }")?;
            return Ok(());
        }
        loop {
            let _json_key = self
                .input
                .next_if(|token| matches!(token, Token::String(_)))
                .ok_or("Expected a string based key")?;
            let _semi_colon = self.input.next_if_eq(&Token::Colon).ok_or("Expected :")?;

            let end_of_content = self.parse_content()?;
            if end_of_content {
                let _closing_bracket = self
                    .input
                    .next_if_eq(&Token::RightBracket)
                    .ok_or("Expected } closing the json")?;
                break;
            }
        }

        Ok(())
    }

    fn parse_content(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.next().is_none() {
            return Err("Unexpected end of file".into());
        }
        let end_of_content = self
            .input
            .next_if(|token| matches!(token, Token::Comma))
            .is_none();
        Ok(end_of_content)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new(tokens)?;
    parser.parse()
}
