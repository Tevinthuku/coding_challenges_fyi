use anyhow::{anyhow, Context};
use itertools::Itertools;
use multipeek::multipeek;

use crate::{db::Db, response::Response};
mod add;
mod append;
pub mod extractors;
mod get;
mod prepend;
mod replace;
mod set;

pub struct Parser {
    content: multipeek::MultiPeek<std::vec::IntoIter<u8>>,
    /// useful for debugging purposes
    full_command: String,
}

impl Parser {
    pub fn new(content: &[u8]) -> Self {
        let content = content
            .iter()
            .copied()
            .filter(|b| !matches!(*b, b'\n' | b'\r'))
            .collect_vec();
        let full_command = String::from_utf8_lossy(&content).to_string();
        Self {
            content: multipeek(content),
            full_command,
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

pub fn execute_command(data: &[u8], db: &Db) -> anyhow::Result<Response> {
    let mut parser = Parser::new(data);
    let full_command = parser.full_command.clone();
    println!("Executing command: {}", full_command);
    let command = parser.next_string().ok_or(anyhow!("Expected a command"))?;
    match command.as_str() {
        "get" => Ok(get::GetCommand::parse(parser)
            .with_context(|| format!("Failed to parse set command: {}", full_command))?
            .execute(db)),

        "set" => Ok(set::SetCommand::parse(parser)
            .with_context(|| format!("Failed to parse set command: {}", full_command))?
            .execute(db)),

        "add" => Ok(add::AddCommand::parse(parser)
            .with_context(|| format!("Failed to parse add command: {}", full_command))?
            .execute(db)),

        "replace" => Ok(replace::ReplaceCommand::parse(parser)
            .with_context(|| format!("Failed to parse replace command: {}", full_command))?
            .execute(db)),

        "append" => Ok(append::AppendCommand::parse(parser)
            .with_context(|| format!("Failed to parse append command: {}", full_command))?
            .execute(db)),

        "prepend" => Ok(prepend::Prepend::parse(parser)
            .with_context(|| format!("Failed to parse prepend command: {}", full_command))?
            .execute(db)),

        cmd => Err(anyhow!("Unknown command {cmd}")),
    }
}
