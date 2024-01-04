#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Token {
    LeftBracket,
    RightBracket,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let input = input.chars();
    input
        .filter_map(|ch| match ch {
            '{' => Some(Token::LeftBracket),
            '}' => Some(Token::RightBracket),
            c if c.is_whitespace() => None,
            _ => unimplemented!(),
        })
        .collect()
}
