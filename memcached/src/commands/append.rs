use crate::{
    db::{Content, Db},
    response::Response,
};

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
            let appended = data.append(&self.data.key, Content::from(&self.data));
            if appended {
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
