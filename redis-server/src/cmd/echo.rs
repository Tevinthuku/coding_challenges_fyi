use super::ParseFrames;
use crate::cmd::anyhow;
use crate::resp::Frame;

pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Frame> {
    let next_string = parser
        .next_string()
        .ok_or_else(|| anyhow!("Expected a string to echo but found None"))?;

    Ok(Frame::new_bulk_string(next_string))
}
