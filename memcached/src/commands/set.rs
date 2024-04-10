use anyhow::{anyhow, Context};

use crate::{
    db::{Content, Db},
    response::Response,
};

use super::Parser;

#[derive(Debug)]
pub struct SetCommand {
    key: String,
    flags: u32,
    exptime: i64,
    bytes: usize,
    noreply: bool,
    content: Vec<u8>,
}

impl SetCommand {
    pub fn parse(mut parser: Parser) -> anyhow::Result<Self> {
        let key = parser.next_string().ok_or(anyhow!("Expected a key"))?;

        let flags = parser
            .next_string()
            .ok_or(anyhow!("Expected a flag"))?
            .parse()
            .context("Failed to parse flags")?;

        let exptime = parser
            .next_string()
            .ok_or(anyhow!("Expected expiry time"))?
            .parse()
            .context("Failed to parse exptime")?;

        let bytes = parser
            .next_string()
            .ok_or(anyhow!("Expected bytes count"))?
            .parse()
            .context("Failed to parse number of bytes")?;

        let maybe_noreply = parser
            .peek_next_string()
            .ok_or(anyhow!("Expected to get noreply or bytes"))?;

        let noreply = if maybe_noreply == "noreply" {
            let _ = parser.next_string();
            true
        } else {
            false
        };

        let content = parser.next_bytes().ok_or(anyhow!("Expected bytes"))?;

        Ok(Self {
            key,
            flags,
            exptime,
            bytes,
            noreply,
            content,
        })
    }

    pub fn execute(self, db: &Db) -> Response {
        use std::cmp::Ordering;

        let exp_duration = match self.exptime.cmp(&0) {
            Ordering::Equal => None,
            // expires immediately
            Ordering::Less => Some(std::time::Duration::from_secs(0)),
            Ordering::Greater => {
                let exptime = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    + std::time::Duration::from_secs(self.exptime as u64);
                Some(exptime)
            }
        };
        db.with_data_mut(|data| {
            data.insert(
                self.key,
                Content {
                    data: self.content,
                    byte_count: self.bytes,
                    flags: self.flags,
                    exp_duration,
                },
            );
            if self.noreply {
                Response::NoReply
            } else {
                Response::Stored
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::Parser;

    use super::SetCommand;

    #[test]
    fn test_set_command() {
        let content = "set test 0 0 4
        1234
        "
        .as_bytes();
        let mut parser = Parser::new(content);
        let _command = parser.next_string().unwrap();
        let set_command = SetCommand::parse(parser).unwrap();
        assert_eq!(set_command.key, "test");
        assert_eq!(set_command.flags, 0);
        assert_eq!(set_command.exptime, 0);
        assert_eq!(set_command.bytes, 4);
        assert!(!set_command.noreply);
        assert_eq!(set_command.content, b"1234".to_vec());
    }
}
