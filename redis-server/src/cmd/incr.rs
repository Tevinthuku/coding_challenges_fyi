use std::{collections::hash_map::Entry, io};

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
        let new_value = db.with_data(|data| {
            let entry = data.entry(self.key.clone());
            let new_val = match entry {
                Entry::Occupied(val) => {
                    let value = val.into_mut();
                    let value = String::from_utf8(value.to_vec())
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
                    match value.parse::<i64>() {
                        Ok(value) => value + 1,
                        Err(_) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "Value is not an integer",
                            ))
                        }
                    }
                }
                Entry::Vacant(_) => 1,
            };

            data.insert(self.key, Bytes::from(format!("{}", new_val)));

            Ok(new_val)
        })?;

        conn.write_frame(Frame::Integer(new_value))
    }
}
