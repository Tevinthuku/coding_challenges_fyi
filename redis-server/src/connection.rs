use std::{
    io::{self, Write},
    net::TcpStream,
};

use anyhow::Context;
use bytes::{BufMut, BytesMut};

use std::io::Read;

use crate::frame::Frame;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self { stream: socket }
    }

    pub fn read_bytes(&mut self) -> anyhow::Result<Option<BytesMut>> {
        let mut data = [0_u8; 128];

        let n = self
            .stream
            .read(&mut data)
            .context("Failed to read buffer")?;

        if n == 0 {
            return Ok(None);
        }

        Ok(Some(BytesMut::from(&data[..n])))
    }

    pub fn send_error(&mut self, err: &str) -> io::Result<()> {
        let frame = Frame::Error(err.to_string());
        self.write_frame(frame)
    }

    pub fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        let response = match frame {
            Frame::SimpleString(content) => {
                let mut response = BytesMut::new();
                response.put_u8(b'+');
                response.extend(content.as_bytes());
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::Error(content) => {
                let mut response = BytesMut::new();
                response.put_u8(b'-');
                response.extend(content.as_bytes());
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::BulkString(bytes) => {
                let mut response = BytesMut::new();
                response.put_u8(b'$');
                response.extend(bytes.len().to_string().as_bytes());
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response.extend(&bytes);
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::Boolean(bool) => {
                let mut response = BytesMut::new();
                response.put_u8(b'#');
                let val = if bool { b't' } else { b'f' };
                response.put_u8(val);
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::Integer(val) => {
                let mut response = BytesMut::new();
                response.put_u8(b':');
                response.extend(val.to_string().as_bytes());
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::Double(val) => {
                let mut response = BytesMut::new();
                response.put_u8(b',');
                self.stream.write_all(&[b','])?;

                if val < 0.0 {
                    response.put_u8(b'-');
                }
                response.extend(val.to_string().as_bytes());
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            Frame::Null => {
                let mut response = BytesMut::with_capacity(3);
                response.put_u8(b'_');
                response.put_u8(b'\r');
                response.put_u8(b'\n');
                response
            }
            _ => unimplemented!(),
        };

        self.stream.write_all(&response)
    }
}
