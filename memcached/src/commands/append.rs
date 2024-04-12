use crate::{db::Db, response::Response};
use linked_hash_map::Entry;

use super::{extractors::ExtractedData, Parser};

pub struct AppendCommand {
    data: ExtractedData,
}

impl AppendCommand {
    pub fn parse(parser: Parser) -> anyhow::Result<Self> {
        let data = ExtractedData::parse(parser)?;

        Ok(Self { data })
    }

    pub fn execute(self, db: &Db) -> Response {
        db.with_data_mut(|data| {
            let entry = data.entry(self.data.key.clone()).and_modify(|content| {
                content.data.extend(self.data.content);
            });
            match entry {
                Entry::Occupied(_) => {
                    if self.data.noreply {
                        Response::NoReply
                    } else {
                        Response::Stored
                    }
                }
                Entry::Vacant(_) => Response::NotStored,
            }
        })
    }
}
