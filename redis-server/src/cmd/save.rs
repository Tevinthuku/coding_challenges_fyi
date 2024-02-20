use std::io;

use crate::{connection::Connection, db::Db};

pub struct Save;

impl Save {
    pub fn execute(&self, conn: &mut Connection, db: &Db) -> io::Result<()> {
        db.save()?;
        conn.write_frame(crate::frame::Frame::SimpleString("OK".to_owned()))
    }
}
