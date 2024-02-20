use std::io;

use super::ParseFrames;
use crate::{cmd::anyhow, connection::Connection, db::Db, frame::Frame};
pub struct Get {
    key: String,
}

impl Get {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow!("Expected a string for key but found None"))?;
        Ok(Self { key })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let frame = db.with_data(|data| {
            data.get(&self.key)
                .map(|value| Frame::BulkString(value.clone()))
                .unwrap_or(Frame::Null)
        });
        conn.write_frame(frame)
    }
}
