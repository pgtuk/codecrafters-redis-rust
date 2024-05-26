use std::io::{self, Cursor};

use bytes::{Buf, BytesMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufWriter}, net::TcpStream};

use super::frame::{Frame, FrameError};


pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {

    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                dbg!(&frame);
                return Ok(Some(frame));
            };

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None)
                } else {
                    return Err("connection reset by peer".into())
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), FrameError> {
        self.write_value(frame).await?;

        self.stream.flush().await?;
        
        Ok(())
    }

    async fn write_value(&mut self, frame: &Frame) -> io::Result<()> {
        self.write_str(frame.to_resp()).await?;

        Ok(())
    }

    async fn write_str(&mut self, string: String) -> io::Result<()> {
        self.stream.write_all(string.as_bytes()).await?;

        Ok(())
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;

                self.buffer.advance(len);

                Ok(Some(frame))
            },
            Err(FrameError::Incomplete) => Ok(None),
            Err(e) => Err(e)
        }

    }
}