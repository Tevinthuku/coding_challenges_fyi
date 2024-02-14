mod echo;
pub mod get;
mod ping;
pub mod set;

use std::io;

use bytes::Bytes;
use log::warn;
use ping::Ping;

use anyhow::{anyhow, bail, Context};

use crate::{connection::Connection, db::Db, frame::Frame};

use self::{echo::Echo, get::Get, set::Set};

pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Unknown,
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Self> {
        let mut parser = ParseFrames::new(match frame {
            Frame::Array(frames) => frames,
            frame => bail!(
                "Expected Array when executing command, but got {}",
                frame.name()
            ),
        });

        let command = parser
            .next_string()
            .context("Failed to parse command")?
            .ok_or_else(|| anyhow!("Expected a command but found nothing"))?
            .to_lowercase();

        match command.as_str() {
            "ping" => Ok(Command::Ping(Ping::parse(&mut parser)?)),
            "echo" => Ok(Command::Echo(Echo::parse(&mut parser)?)),
            "set" => Ok(Command::Set(Set::parse(&mut parser)?)),
            "get" => Ok(Command::Get(Get::parse(&mut parser)?)),
            command => {
                warn!("command: {command}");
                Ok(Command::Unknown)
            }
        }
    }

    pub async fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        match self {
            Command::Ping(ping) => ping.execute(conn).await,
            Command::Echo(echo) => echo.execute(conn).await,
            Command::Set(set) => set.execute(conn, db).await,
            Command::Get(get) => get.execute(conn, db).await,
            Command::Unknown => {
                let frame = Frame::Error("ERR unknown command".to_string());
                conn.write_frame(frame).await
            }
        }
    }
}

pub struct ParseFrames {
    items: std::vec::IntoIter<Frame>,
}

impl ParseFrames {
    fn new(frames: Vec<Frame>) -> Self {
        Self {
            items: frames.into_iter(),
        }
    }

    fn next_string(&mut self) -> anyhow::Result<Option<String>> {
        match self.items.next() {
            Some(Frame::SimpleString(s)) => Ok(Some(s)),
            Some(Frame::BulkString(bytes)) => std::str::from_utf8(&bytes[..])
                .map(|s| Some(s.to_string()))
                .context("Failed to parse string"),
            None => Ok(None),
            _ => {
                bail!("Expected a string but did not find one")
            }
        }
    }

    fn next_bytes(&mut self) -> Option<Bytes> {
        match self.items.next() {
            Some(Frame::SimpleString(s)) => Some(s.into()),
            Some(Frame::BulkString(bytes)) => Some(bytes),
            _ => None,
        }
    }
}
