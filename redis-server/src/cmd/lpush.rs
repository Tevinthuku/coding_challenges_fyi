use std::{collections::VecDeque, io};

use bytes::Bytes;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

use anyhow::anyhow;
pub struct Lpush {
    key: String,
    values: Vec<Bytes>,
}

impl Lpush {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let key = parser
            .next_string()?
            .ok_or_else(|| anyhow!("Expected a string for key but found None"))?;
        let mut values = vec![];
        while let Some(value) = parser.next_bytes()? {
            values.push(value);
        }

        Ok(Self { key, values })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let list = db.with_list_data_mut(self.key, |list| {
            let mut list = VecDeque::from(list);

            for value in self.values {
                list.push_front(value);
            }
            list.into_iter().collect()
        })?;
        let frame = Frame::Integer(list.len() as i64);
        conn.write_frame(frame)
    }
}
