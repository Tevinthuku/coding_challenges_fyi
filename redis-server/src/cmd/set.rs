use std::io;

use bytes::Bytes;

use super::ParseFrames;
use crate::{cmd::anyhow, connection::Connection, db::Db, frame::Frame};

pub struct Set {
    key: String,
    value: Bytes,
}

impl Set {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow!("Expected a string for key but found None"))?;
        let value = parser
            .next_bytes()
            .ok_or_else(|| anyhow!("Expected a string for value but found None"))?;
        Ok(Self { key, value })
    }

    pub async fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        db.set(self.key, self.value);
        let frame = Frame::SimpleString("OK".to_owned());
        conn.write_frame(frame).await
    }
}
