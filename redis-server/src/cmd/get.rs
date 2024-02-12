use std::io;

use super::ParseFrames;
use crate::{cmd::anyhow, connection::Connection, db::Db, frame::Frame};
pub struct Get {
    key: String,
}

impl Get {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()
            .ok_or_else(|| anyhow!("Expected a string for key but found None"))?;
        Ok(Self { key })
    }

    pub async fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        if let Some(value) = db.get(&self.key) {
            let value_as_str = String::from_utf8_lossy(&value);
            let frame = Frame::SimpleString(value_as_str.to_string());
            conn.write_frame(frame).await
        } else {
            let frame = Frame::Null;
            conn.write_frame(frame).await
        }
    }
}
