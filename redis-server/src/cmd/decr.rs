use std::io;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

use anyhow::anyhow;
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
        let new_value = db.with_integer_data_mut(self.key.clone(), |val| val - 1)?;

        conn.write_frame(Frame::Integer(new_value))
    }
}
