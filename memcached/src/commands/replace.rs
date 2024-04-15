use crate::{
    db::{Content, Db},
    response::Response,
};

use super::{extractors::ExtractedData, Parser};

pub struct ReplaceCommand {
    data: ExtractedData,
}

impl ReplaceCommand {
    pub fn parse(parser: Parser) -> anyhow::Result<Self> {
        let data = ExtractedData::parse(parser)?;

        Ok(Self { data })
    }

    pub fn execute(self, db: &Db) -> Response {
        db.with_data_mut(|data| {
            if data.contains_key(&self.data.key) {
                data.insert(self.data.key.clone(), Content::from(&self.data));
                if self.data.noreply {
                    Response::NoReply
                } else {
                    Response::Stored
                }
            } else {
                Response::NotStored
            }
        })
    }
}
