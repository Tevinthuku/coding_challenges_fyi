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
}
