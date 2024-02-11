mod echo;
mod ping;

use std::io;

use ping::Ping;

use anyhow::{anyhow, bail};

use crate::{connection::Connection, frame::Frame};

use self::echo::Echo;

pub enum Command {
    Ping(Ping),
    Echo(Echo),
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
            .ok_or(anyhow!("Expected command but did not find one"))?
            .to_lowercase();

        match command.as_str() {
            "ping" => Ok(Command::Ping(Ping::parse(&mut parser))),
            "echo" => Ok(Command::Echo(Echo::parse(&mut parser)?)),
            _ => unimplemented!(),
        }
    }

    pub async fn execute(self, conn: &mut Connection) -> io::Result<()> {
        match self {
            Command::Ping(ping) => ping.execute(conn).await,
            Command::Echo(echo) => echo.execute(conn).await,
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

    fn next_string(&mut self) -> Option<String> {
        match self.items.next() {
            Some(Frame::SimpleString(s)) => Some(s),
            Some(Frame::BulkString { content, .. }) => Some(content),
            _ => None,
        }
    }
}
