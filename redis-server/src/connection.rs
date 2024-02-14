use std::io;

use anyhow::Context;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use tokio::io::BufWriter;

use crate::frame::Frame;

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

    pub async fn read_bytes(&mut self) -> anyhow::Result<Option<BytesMut>> {
        let n = self
            .stream
            .read_buf(&mut self.buffer)
            .await
            .context("Failed to read buffer")?;

        if n == 0 {
            return Ok(None);
        }

        Ok(Some(self.buffer.clone()))
    }

    pub async fn send_error(&mut self, err: &str) -> io::Result<()> {
        let frame = Frame::Error(err.to_string());
        self.write_frame(frame).await
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
            Frame::BulkString(bytes) => {
                let length = bytes.len();
                self.stream.write_u8(b'$').await?;
                self.stream.write_all(length.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(&bytes).await?;
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
            Frame::Null => {
                self.stream.write_u8(b'_').await?;
                self.stream.write_all(b"\r\n").await?;
            }
            _ => unimplemented!(),
        }

        self.stream.flush().await
    }
}
