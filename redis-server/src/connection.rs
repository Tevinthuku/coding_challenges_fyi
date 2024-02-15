use std::{
    io::{self, BufWriter, Write},
    net::TcpStream,
};

use anyhow::Context;
use bytes::BytesMut;

use std::io::Read;

use crate::frame::Frame;

pub struct Connection {
    stream: BufWriter<TcpStream>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
        }
    }

    pub fn read_bytes(&mut self) -> anyhow::Result<Option<BytesMut>> {
        let mut data = [0_u8; 128];

        let n = self
            .stream
            .get_ref()
            .read(&mut data)
            .context("Failed to read buffer")?;

        if n == 0 {
            return Ok(None);
        }

        Ok(Some(BytesMut::from(&data[..])))
    }

    pub fn send_error(&mut self, err: &str) -> io::Result<()> {
        let frame = Frame::Error(err.to_string());
        self.write_frame(frame)
    }

    pub fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        match frame {
            Frame::SimpleString(content) => {
                self.stream.write_all(&[b'+'])?;
                self.stream.write_all(content.as_bytes())?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::Error(content) => {
                self.stream.write_all(&[b'-'])?;
                self.stream.write_all(content.as_bytes())?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::BulkString(bytes) => {
                let length = bytes.len();
                self.stream.write_all(&[b'$'])?;
                self.stream.write_all(length.to_string().as_bytes())?;
                self.stream.write_all(b"\r\n")?;
                self.stream.write_all(&bytes)?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::Boolean(bool) => {
                self.stream.write_all(&[b'#'])?;
                let val = if bool { b't' } else { b'f' };
                self.stream.write_all(&[val])?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::Integer(val) => {
                self.stream.write_all(&[b':'])?;
                if val < 0 {
                    self.stream.write_all(&[b'-'])?;
                }
                self.stream.write_all(val.to_string().as_bytes())?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::Double(val) => {
                self.stream.write_all(&[b','])?;
                if val < 0.0 {
                    self.stream.write_all(&[b'-'])?;
                }
                self.stream.write_all(val.to_string().as_bytes())?;
                self.stream.write_all(b"\r\n")?;
            }
            Frame::Null => {
                self.stream.write_all(&[b'_', b'\r', b'\n'])?;
            }
            _ => unimplemented!(),
        }

        self.stream.flush()
    }
}
