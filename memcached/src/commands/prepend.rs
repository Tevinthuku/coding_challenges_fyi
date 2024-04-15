use crate::{
    db::{Content, Db},
    response::Response,
};

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
            let prepended = data.prepend(&self.data.key, Content::from(&self.data));
            if prepended {
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
