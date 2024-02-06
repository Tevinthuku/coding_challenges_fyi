use std::iter::Peekable;

#[derive(Debug)]
enum Token {
    SimpleString(String),
    // \r\n
    CRLF,
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
            println!("{ch}");
            match ch {
                '\r' if matches!(self.0.peek(), Some('\n')) => {
                    let _dash_n = self.0.next();
                }
                ch => {
                    result.push(ch);
                }
            }
        }
        result
    }
    fn deserialize(mut self) -> Option<Token> {
        let ch = self.0.next()?;

        let content = match ch {
            '+' => Token::SimpleString(self.string()),
            _ => todo!(),
        };
        Some(content)
    }
}

#[cfg(test)]
mod tests {
    use super::Tokenizer;

    #[test]
    fn test_content_section() {
        let tokenizer = Tokenizer::new("+OK\r\n");
        let result = tokenizer.deserialize().unwrap();
        println!("{result:?}");
    }
}
