use std::io;

use crate::{connection::Connection, db::Db, frame::Frame};

use super::ParseFrames;

pub struct Del {
    keys: Vec<String>,
}

impl Del {
    pub fn parse(parser: &mut ParseFrames) -> anyhow::Result<Self> {
        let mut keys = vec![];
        while let Some(key) = parser.next_string()? {
            keys.push(key);
        }
        Ok(Self { keys })
    }

    pub fn execute(self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        let delete_count = db.with_data(|data| {
            self.keys.iter().fold(0, |acc, key| {
                if data.remove(key).is_some() {
                    acc + 1
                } else {
                    acc
                }
            })
        });
        let frame = Frame::Integer(delete_count);
        conn.write_frame(frame)
    }
}
