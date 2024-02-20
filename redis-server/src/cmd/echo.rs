use std::io;

use bytes::Bytes;

use super::ParseFrames;
use crate::frame::Frame;
use crate::{cmd::anyhow, connection::Connection};

pub struct Echo {
    message: Bytes,
}

impl Echo {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let message = parser
            .next_bytes()?
            .ok_or_else(|| anyhow!("Expected a string to echo but found None"))?;
        Ok(Self { message })
    }

    pub fn execute(self, conn: &mut Connection) -> io::Result<()> {
        let frame = Frame::new_bulk_string(self.message);

        conn.write_frame(frame)
    }
}
