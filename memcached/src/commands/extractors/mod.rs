use anyhow::Context;

use crate::{anyhow, db::Content};
use std::cmp::Ordering;
use std::time::Duration;

use super::Parser;
#[derive(Debug)]
pub struct ExtractedData {
    pub key: String,
    pub flags: u32,
    pub exptime: Option<Duration>,
    pub bytes: usize,
    pub noreply: bool,
    pub content: Vec<u8>,
}

impl ExtractedData {
    pub fn parse(mut parser: Parser) -> anyhow::Result<Self> {
        let key = parser.next_string().ok_or(anyhow!("Expected a key"))?;

        let flags = parser
            .next_string()
            .ok_or(anyhow!("Expected a flag"))?
            .parse()
            .context("Failed to parse flags")?;

        let exptime_in_sec = parser
            .next_string()
            .ok_or(anyhow!("Expected expiry time"))?
            .parse::<i64>()
            .context("Failed to parse exptime")?;

        let exptime = match exptime_in_sec.cmp(&0) {
            Ordering::Equal => None,
            // expires immediately
            Ordering::Less => Some(std::time::Duration::from_secs(0)),
            Ordering::Greater => {
                let exptime = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    + std::time::Duration::from_secs(exptime_in_sec as u64);
                Some(exptime)
            }
        };

        let bytes = parser
            .next_string()
            .ok_or(anyhow!("Expected bytes count"))?
            .parse()
            .context("Failed to parse number of bytes")?;

        let maybe_noreply = parser
            .peek_next_string()
            .ok_or(anyhow!("Expected to get noreply or bytes"))?;

        let noreply = if maybe_noreply == "noreply" {
            let _ = parser.next_string();
            true
        } else {
            false
        };

        let content = parser.next_bytes().ok_or(anyhow!("Expected bytes"))?;

        Ok(Self {
            key,
            flags,
            exptime,
            bytes,
            noreply,
            content,
        })
    }
}

impl From<&ExtractedData> for Content {
    fn from(value: &ExtractedData) -> Self {
        Self {
            data: value.content.clone(),
            byte_count: value.bytes,
            flags: value.flags,
            exp_duration: value.exptime,
        }
    }
}
