use std::mem;

use itertools::Itertools;
use linked_hash_map::Entry;

use crate::{db::Db, response::Response};

use super::{extractors::ExtractedData, Parser};

pub struct Prepend {
    data: ExtractedData,
}

impl Prepend {
    pub fn parse(parser: Parser) -> anyhow::Result<Self> {
        let data = ExtractedData::parse(parser)?;

        Ok(Self { data })
    }

    pub fn execute(self, db: &Db) -> Response {
        db.with_data_mut(|data| {
            let entry = data.entry(self.data.key.clone()).and_modify(|content| {
                let existing_content = mem::take(&mut content.data);
                content.data = self
                    .data
                    .content
                    .into_iter()
                    .chain(existing_content)
                    .collect_vec();
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
