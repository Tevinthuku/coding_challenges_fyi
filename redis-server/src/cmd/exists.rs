use std::io;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

pub struct Exists {
    keys: Vec<String>,
}

impl Exists {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let mut keys = vec![];
        while let Some(key) = parser.next_string()? {
            keys.push(key);
        }
        Ok(Self { keys })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let count = db.with_data(|data| {
            data.iter().fold(0, |acc, (key, _)| {
                if self.keys.contains(key) {
                    acc + 1
                } else {
                    acc
                }
            })
        });
        let frame = Frame::Integer(count);
        conn.write_frame(frame)
    }
}
