use super::ParseFrames;
use crate::resp::Frame;

pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Frame> {
    let optional_message = parser.next_string();

    if let Some(message) = optional_message {
        Ok(Frame::new_bulk_string(message))
    } else {
        Ok(Frame::SimpleString("PONG".to_string()))
    }
}
