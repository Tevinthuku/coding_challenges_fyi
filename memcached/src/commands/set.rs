use crate::{
    db::{Content, Db},
    response::Response,
};

use super::{extractors::ExtractedData, Parser};

#[derive(Debug)]
pub struct SetCommand {
    data: ExtractedData,
}

impl SetCommand {
    pub fn parse(parser: Parser) -> anyhow::Result<Self> {
        let data = ExtractedData::parse(parser)?;

        Ok(Self { data })
    }

    pub fn execute(self, db: &Db) -> Response {
        db.with_data_mut(|data| {
            data.insert(self.data.key.clone(), Content::from(&self.data));
            if self.data.noreply {
                Response::NoReply
            } else {
                Response::Stored
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::Parser;

    use super::SetCommand;

    #[test]
    fn test_set_command() {
        let content = "set test 0 0 4
        1234
        "
        .as_bytes();
        let mut parser = Parser::new(content);
        let _command = parser.next_string().unwrap();
        let set_command_data = SetCommand::parse(parser).unwrap().data;
        assert_eq!(set_command_data.key, "test");
        assert_eq!(set_command_data.flags, 0);
        assert_eq!(set_command_data.exptime, None);
        assert_eq!(set_command_data.bytes, 4);
        assert!(!set_command_data.noreply);
        assert_eq!(set_command_data.content, b"1234".to_vec());
    }
}
