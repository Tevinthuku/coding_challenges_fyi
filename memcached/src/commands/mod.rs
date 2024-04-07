use itertools::Itertools;
use multipeek::multipeek;
mod set;
pub mod get;

pub struct Parser {
    content: multipeek::MultiPeek<std::vec::IntoIter<u8>>,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        let content = content
            .into_iter()
            .filter(|b| !matches!(*b, b'\n' | b'\r'))
            .collect_vec();
        Self {
            content: multipeek(content),
        }
    }

    pub fn next_string(&mut self) -> Option<String> {
        let (content, skipped) = self.peek_next_bytes()?;
        for _ in 0..skipped {
            self.content.next();
        }
        String::from_utf8(content).ok()
    }

    pub fn peek_next_string(&mut self) -> Option<String> {
        let (content, _) = self.peek_next_bytes()?;
        String::from_utf8(content).ok()
    }

    pub fn next_bytes(&mut self) -> Option<Vec<u8>> {
        let (content, counter) = self.peek_next_bytes()?;
        for _ in 0..counter {
            self.content.next();
        }
        Some(content)
    }

    pub fn peek_next_bytes(&mut self) -> Option<(Vec<u8>, usize)> {
        let mut skip_white_space_counter = 0;

        loop {
            let byte = self.content.by_ref().peek_nth(skip_white_space_counter)?;
            if *byte != b' ' {
                break;
            }
            skip_white_space_counter += 1;
        }
        let mut counter = skip_white_space_counter;
        let mut buf = Vec::new();

        loop {
            let byte = self.content.peek_nth(counter);
            match byte {
                None => break,
                Some(byte) => {
                    if *byte == b' ' {
                        break;
                    }
                    buf.push(*byte);
                    counter += 1;
                }
            }
        }

        Some((buf, counter))
    }
}
