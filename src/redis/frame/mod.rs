use bytes::{Buf, Bytes};
use thiserror::Error;
use std::{
    self,
    fmt, 
    io::Cursor,
    num::{ParseIntError, TryFromIntError},
    string::FromUtf8Error, vec,
};

use crate::redis::utils::{add_cr, int_as_bytes};


#[derive(Debug, PartialEq, Clone)]
pub enum Frame {
    Simple(String),
    Bulk(Bytes),
    Integer(u64),
    Null,
    Array(Vec<Frame>),
}

#[derive(Error, Debug)]
pub enum FrameError {
    Incomplete,
    Other(String),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        match get_u8(src)? {
            // simple
            b'+' => {
                get_line(src)?;
            },
            // bulk
            b'$' => {
                if peek(src)? == b'-' {
                    
                    skip(src, 4)?
                } else {
                    let len = get_int(src)?;

                    // check for valid string of len `len` + \r\n
                    skip(src, len + 2)?
                }
            },
            //integer
            b':' => {
                get_int(src)?;
            },
            // array
            b'*' => {
                let len = get_int(src)?;
                for _ in 0..len {
                    Frame::check(src)?;
                }
            },
            unknown => {
                return Err(
                    FrameError::Other(
                        format!("protocol error; invalid frame type byte `{}`", unknown)
                    )
                );
            }
        }
      
        Ok(())
    }
    pub fn parse_rdb(src: &mut Cursor<&[u8]>) -> Result<Bytes, FrameError> {
        match get_u8(src)? {
            b'$' => {
                let content_len = get_int(src)?;
                let start = src.position() as usize;
                if src.get_ref().len() - 1 < start + content_len {
                    return Err(FrameError::Other("Corrupted RDB".into()))
                }
                src.set_position((start + content_len) as u64);

                let resp = Ok(src.get_ref()[start..start + content_len].to_vec().into());

                resp
            },
            _ => {
                Err(FrameError::Other("Wrong RDB file format".into()))
            }
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(src)? {
            // simple
            b'+' => {
                let line = get_line(src)?.to_vec();

                let string = String::from_utf8(line)?;
                
                Ok(Frame::Simple(string))
            },
            // bulk
            b'$' => {
                if peek(src)? == b'-' {
                    let line = get_line(src)?;

                    if line != b"-1" {
                        return Err(
                            FrameError::Other("Wrong format".to_string())
                        );
                    }

                    Ok(Frame::Null)
                } else {
                    let len = get_int(src)?;

                    if src.remaining() < len + 2 {
                        return Err(FrameError::Incomplete);
                    }
                    
                    let line = get_line(src)?.to_vec();

                    Ok(Frame::Bulk(line.into()))
                }
            },
            // integer
            b':' => {
                let int = get_int(src)?;

                Ok(Frame::Integer(int.try_into()?))
            },
            // array
            b'*' => {
                let len = get_int(src)?;

                let mut result = Vec::with_capacity(len);

                for _ in 0..len {
                    result.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(result))
            },
            // unknown
            any => {
                eprintln!("Unknown frame type: {}", String::from_utf8(vec![any]).unwrap());
                unimplemented!()
            }
        }
    }

    pub fn to_response(&self) -> Vec<u8> {
        match self {
            Frame::Simple(val) => {
                format!("+{}\r\n", val)
                    .as_bytes()
                    .to_vec()
            },
            Frame::Null => {
                b"$-1\r\n".to_vec()
            },
            Frame::Integer(val) => {
                format!(":{}\r\n", val).as_bytes().to_vec()
            },
            Frame::Bulk(val) => {
                let mut buff: Vec<u8> = Vec::new();

                buff.push(b'$');

                buff.extend(int_as_bytes(&val.len()));
                add_cr(&mut buff);
                buff.extend(val);
                add_cr(&mut buff);

                buff
            },
            Frame::Array(arr) => {
                let mut buff: Vec<u8> = Vec::new();
                buff.push(b'*');

                buff.extend(int_as_bytes(&arr.len()));

                add_cr(&mut buff);
                for frame in arr {
                    buff.extend(frame.to_response());
                }

                buff
            }
        }
    }

    pub fn array() -> Frame {
        Frame::Array(vec![])
    }

    pub fn add(&mut self, frame: Frame) {
        match self {
            Frame::Array(arr) => arr.push(frame),
            _ => panic!("Must be an array frame")
        }
    }
}


fn peek(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if src.remaining() < n {
        return Err(FrameError::Incomplete)
    }

    src.advance(n);
    Ok(())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete)
    }
    
    Ok(src.get_u8())
}

fn get_int(src: &mut Cursor<&[u8]>) -> Result<usize, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete)
    }

    String::from_utf8(
        get_line(src)?.to_vec()
    )?
    .parse()
    .map_err(|_| {
        FrameError::Other("Can't parse integer".to_string())
    })
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..i])
        }
    }
    Err(FrameError::Incomplete)
}

impl From<String> for FrameError {
    fn from(src: String) -> FrameError {
        FrameError::Other(src)
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
        FrameError::Other(format!("{:?}", value))
    }
}

impl From<ParseIntError> for FrameError {
    fn from(_: ParseIntError) -> FrameError {
        FrameError::Other("Unable to parse int".into())
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::Incomplete => "stream ended early".fmt(fmt),
            FrameError::Other(err) => err.fmt(fmt),
        }
    }
}

#[cfg(test)]
mod tests;
