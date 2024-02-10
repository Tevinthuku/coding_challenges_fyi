use super::ParseFrames;
use crate::cmd::anyhow;
use crate::resp::Frame;

pub struct Echo {
    message: String,
}

impl Echo {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let message = parser
            .next_string()
            .ok_or_else(|| anyhow!("Expected a string to echo but found None"))?;
        Ok(Self { message })
    }

    pub fn execute(self) -> Frame {
        Frame::new_bulk_string(self.message)
    }
}
