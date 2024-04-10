use crate::db::Content;

#[derive(Debug)]
pub struct ValueResponse {
    pub key: String,
    pub flags: u32,
    pub data: Vec<u8>,
    byte_count: usize,
}
#[derive(Debug)]
pub enum Response {
    Stored,
    NoReply,
    End,
    Value(ValueResponse),
    Error(String),
}

impl From<(&Content, String)> for ValueResponse {
    fn from((content, key): (&Content, String)) -> Self {
        Self {
            key,
            flags: content.flags,
            data: content.data.clone(),
            byte_count: content.byte_count,
        }
    }
}

impl Response {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Response::Stored => b"STORED\r\n".to_vec(),
            Response::NoReply => Vec::new(),
            Response::End => b"END\r\n".to_vec(),
            Response::Value(value) => {
                let mut bytes = Vec::new();
                let response = format!(
                    "VALUE {} {} {}\r\n",
                    value.key, value.flags, value.byte_count,
                );
                bytes.extend_from_slice(response.as_bytes());
                bytes.extend_from_slice(&value.data);
                bytes.extend_from_slice(b"\r\n");
                bytes.extend_from_slice("END\r\n".as_bytes());
                bytes
            }
            Response::Error(message) => format!("ERROR {}\r\n", message).into_bytes(),
        }
    }
}
