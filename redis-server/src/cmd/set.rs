use std::{io, time::Duration};

use anyhow::Context;
use bytes::Bytes;

use super::ParseFrames;
use crate::{cmd::anyhow, connection::Connection, db::Db, frame::Frame};

#[derive(Default)]
struct Options {
    expiration: Option<Duration>,
    return_existing_value: bool,
}

pub struct Set {
    key: String,
    value: Bytes,
    options: Options,
}

impl Set {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow!("Expected a string for key but found None"))?;
        let value = parser
            .next_bytes()?
            .ok_or_else(|| anyhow!("Expected a string for value but found None"))?;

        let options = Self::options(parser)?;
        Ok(Self {
            key,
            value,
            options,
        })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let old_value = db.set(self.key, self.value, self.options.expiration);

        let frame_to_write = if self.options.return_existing_value {
            old_value.map(Frame::BulkString).unwrap_or(Frame::Null)
        } else {
            Frame::SimpleString("OK".to_owned())
        };

        conn.write_frame(frame_to_write)
    }

    fn options(parser: &mut ParseFrames) -> anyhow::Result<Options> {
        let mut options = Options::default();
        while let Some(option) = parser.next_string()? {
            let option = option.to_lowercase();
            match option.as_str() {
                "ex" | "px" => {
                    let duration = parser
                        .next_bytes()?
                        .ok_or_else(|| anyhow!("Expected a value for duration but found None"))?;
                    let duration = serde_json::from_slice(&duration)
                        .context("Failed to parse time to a positive integer")?;

                    match option.as_str() {
                        "px" => {
                            options.expiration = Some(Duration::from_millis(duration));
                        }
                        "ex" => {
                            options.expiration = Some(Duration::from_secs(duration));
                        }
                        _ => {}
                    }
                }
                "get" => {
                    options.return_existing_value = true;
                }

                _ => {}
            }
        }

        Ok(options)
    }
}
