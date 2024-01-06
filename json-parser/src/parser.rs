use std::{error::Error, iter::Peekable};

use crate::tokenizer::Token;

const EMPTY_JSON_TOKEN_COUNT: usize = 2;

struct Parser {
    input: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Result<Self, &'static str> {
        if tokens.len() < EMPTY_JSON_TOKEN_COUNT {
            return Err("Invalid json. Not enough tokens provided");
        }
        Ok(Self {
            input: tokens.into_iter().peekable(),
        })
    }

    fn next(&mut self) -> Option<Token> {
        self.input.next()
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let _opening_bracket = self
            .input
            .next_if_eq(&Token::LeftCurlyBracket)
            .ok_or("Expected {")?;
        self.parse_object_content()
    }

    fn parse_object_content(&mut self) -> Result<(), Box<dyn Error>> {
        if self.input.next_if_eq(&Token::RightCurlyBracket).is_some() {
            return Ok(());
        }
        loop {
            let _json_key = self
                .input
                .next_if(|token| matches!(token, Token::String(_)))
                .ok_or("Expected a String based key")?;

            let _semi_colon = self.input.next_if_eq(&Token::Colon).ok_or("Expected :")?;

            self.parse_value_content()?;

            let maybe_comma_or_closing_bracket = self.next().ok_or("Unexpected end of file")?;

            match maybe_comma_or_closing_bracket {
                Token::Comma => {
                    continue;
                }
                Token::RightCurlyBracket => {
                    break;
                }
                token => {
                    return Err(format!(
                        "Expected either comma, or closing bracket, but found {token:?}"
                    )
                    .into())
                }
            }
        }
        Ok(())
    }

    fn parse_value_content(&mut self) -> Result<(), Box<dyn Error>> {
        let token = self.next().ok_or("Unexpected end of file")?;
        match token {
            Token::LeftSquareBracket => {
                self.parse_array()?;
            }
            Token::Boolean | Token::Digit | Token::String(_) | Token::Null => {}
            Token::LeftCurlyBracket => self.parse_object_content()?,
            token => {
                return Err(
                    format!("Unexpected token while parsing value content {token:?}").into(),
                )
            }
        }

        Ok(())
    }

    fn parse_array(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let peeked = self.input.peek().ok_or("Unexpected end of input")?;
            if peeked == &Token::RightSquareBracket {
                break;
            } else {
                self.parse_value_content()?;
            }
        }

        let _closing_bracket = self
            .input
            .next_if_eq(&Token::RightSquareBracket)
            .ok_or("Expected ] to close the array")?;

        Ok(())
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new(tokens)?;
    parser.parse()
}
