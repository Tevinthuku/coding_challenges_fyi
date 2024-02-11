use std::io;

use anyhow::{bail, Context};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use tokio::io::BufWriter;

use crate::resp::Frame;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read(&mut self) -> anyhow::Result<Option<String>> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            if self.buffer.is_empty() {
                return Ok(None);
            } else {
                bail!("Connection reset by peer")
            }
        }

        let content = self.buffer.to_vec();
        let content =
            String::from_utf8(content).context("Failed to convert data to readable format")?;

        Ok(Some(content))
    }

    pub async fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        match frame {
            Frame::SimpleString(content) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(content.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(content) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(content.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::BulkString { content, length } => {
                self.stream.write_u8(b'$').await?;
                self.stream.write_all(length.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(content.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Boolean(bool) => {
                self.stream.write_u8(b'#').await?;
                self.stream.write_u8(if bool { b't' } else { b'f' }).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                if val < 0 {
                    self.stream.write_u8(b'-').await?;
                }
                self.stream.write_all(val.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Double(val) => {
                self.stream.write_u8(b',').await?;
                if val < 0.0 {
                    self.stream.write_u8(b'-').await?;
                }
                self.stream.write_all(val.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            _ => unimplemented!(),
        }

        self.stream.flush().await
    }
}
