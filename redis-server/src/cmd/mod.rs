pub mod del;
mod echo;
pub mod exists;
pub mod get;
pub mod incr;
mod ping;
pub mod set;

use std::io;

use bytes::Bytes;
use log::warn;
use ping::Ping;

use anyhow::{anyhow, bail, Context};

use crate::{connection::Connection, db::Db, frame::Frame};

use self::{del::Del, echo::Echo, exists::Exists, get::Get, incr::Incr, set::Set};

pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Exists(Exists),
    Del(Del),
    Incr(Incr),
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
            "exists" => Ok(Command::Exists(Exists::parse(&mut parser)?)),
            "del" => Ok(Command::Del(Del::parse(&mut parser)?)),
            "incr" => Ok(Command::Incr(Incr::parse(&mut parser)?)),
            command => {
                warn!("command: {command}");
                Ok(Command::Unknown)
            }
        }
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        match self {
            Command::Ping(ping) => ping.execute(conn),
            Command::Echo(echo) => echo.execute(conn),
            Command::Set(set) => set.execute(conn, db),
            Command::Get(get) => get.execute(conn, db),
            Command::Exists(exists) => exists.execute(conn, db),
            Command::Del(del) => del.execute(conn, db),
            Command::Incr(incr) => incr.execute(conn, db),
            Command::Unknown => {
                let frame = Frame::Error("ERR unknown command".to_string());
                conn.write_frame(frame)
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
