use std::io;

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
        let new_value = db.with_integer_data_mut(self.key.clone(), |val| val + 1)?;
        conn.write_frame(Frame::Integer(new_value))
    }
}
