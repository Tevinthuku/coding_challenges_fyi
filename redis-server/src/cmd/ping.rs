use super::ParseFrames;
use crate::resp::Frame;

pub struct Ping {
    optional_message: Option<String>,
}

impl Ping {
    pub fn parse(parser: &mut ParseFrames) -> Self {
        let optional_message = parser.next_string();
        Self { optional_message }
    }

    pub fn execute(self) -> Frame {
        if let Some(message) = self.optional_message {
            Frame::SimpleString(message)
        } else {
            Frame::SimpleString("PONG".to_string())
        }
    }
}
