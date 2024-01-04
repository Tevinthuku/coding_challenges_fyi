use std::{error::Error, iter::Peekable};

use crate::tokenizer::Token;

struct Parser {
    count_of_tokens: usize,
    input: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            count_of_tokens: tokens.len(),
            input: tokens.into_iter().peekable(),
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.input.next()
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let _opening_bracket = self
            .input
            .next_if_eq(&Token::LeftBracket)
            .ok_or("Expected {")?;
        // count_of_tokens == 2 symbolizes that we only have 2 tokens ['{', '}']
        // Its possible that the 2 tokens might not be the opening and closing brackets
        // but this will be caught above at the _opening_bracket check and at the beginning of the loop for the closing bracket at the end_of_json check.
        let mut end_of_content = self.count_of_tokens == 2;
        loop {
            let end_of_json = self.input.next_if_eq(&Token::RightBracket);
            if end_of_json.is_some() && end_of_content {
                return Ok(());
            }
            let _json_key = self
                .input
                .next_if(|token| matches!(token, Token::String(_)))
                .ok_or("Expected a string based key")?;
            let _semi_colon = self.input.next_if_eq(&Token::Colon).ok_or("Expected :")?;

            end_of_content = self.parse_content()?;
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
        match self.next() {
            Some(Token::String(_)) => {}
            Some(token) => return Err(format!("Did not expect {token:?}").into()),
            None => return Err("Unexpected end of file".into()),
        }
        let end_of_content = self
            .input
            .next_if(|token| matches!(token, Token::Comma))
            .is_none();
        Ok(end_of_content)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
