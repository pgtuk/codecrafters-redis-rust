use std::{
    fmt,
    str, 
    vec,
};
use bytes::Bytes;
use thiserror::Error;

use super::frame::Frame;


#[derive(Debug)]
pub(crate) struct Parser {
    frames: vec::IntoIter<Frame>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    EndOfStream,

    Other(String),
}

impl Parser {
    pub fn new(frame: Frame) -> Result<Parser, ParserError> {
        let frame_array = match frame {
            Frame::Array(array) => array.into_iter(),
            _ => return Err(
                ParserError::Other(
                    format!("protocol error; expected array, got {:?}", frame)
                )
            ),
        };

        Ok(
            Parser { frames: frame_array.into_iter() }
        )
    }

    fn next(&mut self) -> Result<Frame, ParserError> {
        self.frames.next().ok_or(ParserError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, ParserError> {
        match self.next()? {
            Frame::Simple(val) => Ok(val),
            Frame::Bulk(val) => str::from_utf8(&val[..])
                .map(|s| s.to_string())
                .map_err(|_| ParserError::Other("protocol error; invalid string".to_string())),
            frame => {
                Err(
                    ParserError::Other(
                        format!("protocol error; expected simple/bulk frame, got {:?}", frame)
                    )
                )
            }
        }
    }

    pub fn next_bytes(&mut self) -> Result<Bytes, ParserError> {
        match self.next()? {
            Frame::Simple(val) => Ok(val.into()),
            Frame::Bulk(val) => Ok(val),
            frame => {
                Err(
                    ParserError::Other(
                        format!("protocol error; expected simple/bulk frame, got {:?}", frame)
                    )
                )
            }
        }
    }
}

impl From<String> for ParserError {
    fn from(src: String) -> ParserError {
        ParserError::Other(src)
    }
}

impl From<&str> for ParserError {
    fn from(src: &str) -> ParserError {
        src.to_string().into()
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParserError::Other(err) => err.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests;
