mod echo;
mod ping;

use anyhow::{anyhow, bail};

use crate::resp::Frame;

pub fn execute_command(frame: Frame) -> anyhow::Result<Frame> {
    println!("{frame:?}");
    let frames = match frame {
        Frame::Array(frames) => frames,
        frame => {
            bail!(
                "Expected Array when executing command, but got {}",
                frame.name()
            )
        }
    };
    let mut parser = ParseFrames::new(frames);
    let command = parser
        .next_string()
        .ok_or(anyhow!("Expected command but did not find one"))?
        .to_lowercase();

    match command.as_str() {
        "ping" => ping::parse(&mut parser),
        "echo" => echo::parse(&mut parser),
        _ => unimplemented!(),
    }
}

pub(crate) struct ParseFrames {
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
