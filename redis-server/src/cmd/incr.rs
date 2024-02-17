use std::io;

use bytes::Bytes;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

pub struct Incr {
    key: String,
}

impl Incr {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow::anyhow!("No key specified"))?;
        Ok(Self { key })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let new_value = db.with_integer(self.key.clone(), |val, data| {
            let new_val = val + 1;
            data.insert(self.key, Bytes::from(format!("{}", new_val)));
            new_val
        })?;
        conn.write_frame(Frame::Integer(new_value))
    }
}
