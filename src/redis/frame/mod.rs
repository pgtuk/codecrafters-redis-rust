use bytes::Buf;
use std::{
    fmt, 
    io::Cursor,
    num::TryFromIntError,
    string::FromUtf8Error,
};

use super::ProtocolError;

#[derive(Debug)]
pub enum Frame {
    Simple(String),
    Bulk(String),
    // Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum FrameError {
    Incomplete,
    Other(ProtocolError),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        match get_u8(src)? {
            b'+' => {
                get_line(src)?;
                Ok(())
            },
            unknown => Err(
                format!("protocol error; invalid frame type byte `{}`", unknown).into()
            ),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(src)? {
            b'+' => {
                let line = get_line(src)?.to_vec();

                let string = String::from_utf8(line)?;
                
                let frame = Frame::Simple(string);
                
                dbg!(&frame);
                Ok(frame)

                // Ok(Frame::Simple(string))
            },
            _ => unimplemented!()
        }
    }

    pub fn to_resp(&self) -> String {
        match self {
            Frame::Simple(val) => {
                format!(
                     "+{data}\r\n",
                     data=val,
                 )
                 
             },
            Frame::Bulk(val) => {
                format!(
                    "${len}\r\n{data}\r\n", 
                    len=val.len(), 
                    data=val,
                )
            },
            // Frame::Array(_) => unimplemented!()
        }
    }
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete)
    }

    Ok(src.get_u8())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    let line = &src.get_ref()[end-1..end+1];

    dbg!(String::from_utf8(line.to_vec())?);

    for i in start..end {
        if src.get_ref()[i+1] == b'\n' {
        
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..end])
        }
    }
    Err(FrameError::Incomplete)
}

impl From<String> for FrameError {
    fn from(src: String) -> FrameError {
        FrameError::Other(src.into())
    }
}

impl From<&str> for FrameError {
    fn from(src: &str) -> FrameError {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for FrameError {
    fn from(_src: FromUtf8Error) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for FrameError {
    fn from(_src: TryFromIntError) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}

impl From<std::io::Error> for FrameError {
    fn from(value: std::io::Error) -> FrameError {
        value.into()
    }
}

impl std::error::Error for FrameError {}

impl fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::Incomplete => "stream ended early".fmt(fmt),
            FrameError::Other(err) => err.fmt(fmt),
        }
    }
}