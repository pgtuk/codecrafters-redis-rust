use std::io::Cursor;

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
        BufStream
    },
    net::TcpStream,
};

pub(crate) use handler::Handler;

use crate::redis::utils::{add_cr, int_as_bytes};

use super::frame::{Frame, FrameError};

pub(crate) mod handler;
#[derive(Debug)]
pub struct Connection {
    pub stream: BufStream<TcpStream>,
    pub buffer: BytesMut,
    pub(crate) is_repl_conn: bool,
}


impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufStream::new(stream),
            buffer: BytesMut::with_capacity(4096),
            is_repl_conn: false,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            };

            if self.buf_empty().await? {
                return Ok(None);
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), FrameError> {
        self.stream.write_all(&frame.to_response()).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub async fn write_rdb(&mut self, rdb: &Vec<u8>) -> Result<(), FrameError> {
        let mut buff: Vec<u8> = Vec::new();
        buff.push(b'$');
        buff.extend(int_as_bytes(&rdb.len()));
        add_cr(&mut buff);
        buff.extend(rdb);

        self.stream.write_all(&buff).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub async fn read_rdb(&mut self) -> Result<Option<Bytes>, FrameError> {
        loop {
            if let Ok(arr) = self.parse_rdb() {
                return Ok(Some(arr));
            };

            if self.buf_empty().await? {
                return Ok(None);
            }
        }
    }

    fn parse_rdb(&mut self) -> Result<Bytes, FrameError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        let rdb = Frame::parse_rdb(&mut buf)?;
        let len = buf.position() as usize;

        self.buffer.advance(len);

        Ok(rdb)
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;

                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(FrameError::Incomplete) => Ok(None),
            Err(e) => Err(e)
        }
    }

    async fn buf_empty(&mut self) -> Result<bool, FrameError> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            if self.buffer.is_empty() || self.is_repl_conn {
                return Ok(true);
            } else {
                return Err("connection reset by peer".into());
            }
        }

        Ok(false)
    }
}
