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
        db.with_data(|data| {
            let content = data.get(&self.key);
            match content {
                Some(content) if !content.is_expired() => {
                    Response::Value((content, self.key).into())
                }
                _ => Response::End,
            }
        })
    }
}
