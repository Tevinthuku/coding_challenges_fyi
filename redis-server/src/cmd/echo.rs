use std::io;

use super::ParseFrames;
use crate::resp::Frame;
use crate::{cmd::anyhow, connection::Connection};

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

    pub async fn execute(self, conn: &mut Connection) -> io::Result<()> {
        let frame = Frame::new_bulk_string(self.message);

        conn.write_frame(frame).await
    }
}
