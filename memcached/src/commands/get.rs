use crate::{db::Db, response::Response};

use super::Parser;
use anyhow::anyhow;

pub struct GetCommand {
    key: String,
}

impl GetCommand {
    pub fn parse(mut parser: Parser) -> anyhow::Result<Self> {
        let key = parser.next_string().ok_or(anyhow!("Expected a key"))?;
        Ok(Self { key })
    }

    pub fn execute(self, db: &Db) -> Response {
        let content = db.get(&self.key);
        content
            .as_ref()
            .map(|content| Response::Value((content, self.key).into()))
            .unwrap_or(Response::End)
    }
}
