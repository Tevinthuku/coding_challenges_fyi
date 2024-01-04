use std::error::Error;

use crate::tokenizer::Token;

pub fn parse(tokens: Vec<Token>) -> Result<(), Box<dyn Error>> {
    let mut iter = tokens.iter().peekable();

    iter.next_if(|&token| token == &Token::LeftBracket)
        .ok_or("Expected {")?;

    let last_element = iter.last().ok_or("No closing } on json input")?;

    if last_element != &Token::RightBracket {
        return Err(format!("Got {last_element:?} instead of }}").into());
    }
    Ok(())
}
