use std::io;

use super::ParseFrames;
use crate::{connection::Connection, resp::Frame};

pub struct Ping {
    optional_message: Option<String>,
}

impl Ping {
    pub fn parse(parser: &mut ParseFrames) -> Self {
        let optional_message = parser.next_string();
        Self { optional_message }
    }

    pub async fn execute(self, conn: &mut Connection) -> io::Result<()> {
        let frame = if let Some(message) = self.optional_message {
            Frame::SimpleString(message)
        } else {
            Frame::SimpleString("PONG".to_string())
        };
        conn.write_frame(frame).await
    }
}
