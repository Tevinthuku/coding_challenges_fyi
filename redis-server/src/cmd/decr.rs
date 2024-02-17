use std::io;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

use anyhow::anyhow;
use bytes::Bytes;
pub struct Decr {
    key: String,
}

impl Decr {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow!("Expected a key for decrementing, but did not find one"))?;
        Ok(Self { key })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let new_value = db.with_integer(self.key.clone(), |val, data| {
            let new_val = val - 1;
            data.insert(self.key, Bytes::from(format!("{}", new_val)));
            new_val
        })?;

        conn.write_frame(Frame::Integer(new_value))
    }
}
