use bytes::Buf;
use std::{
    fmt, 
    io::Cursor,
    num::{ParseIntError, TryFromIntError},
    string::FromUtf8Error,
};

use super::ProtocolError;

#[derive(Debug, PartialEq)]
pub enum Frame {
    Simple(String),
    Bulk(String),
    Array(Vec<Frame>),

    // NullBulk
    // Integer
    // Error
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
            b'$' => {
                get_line(src)?;
                Ok(())
            },
            b'*' => {
                let len = get_int(src)?;

                for _ in 0..len {
                    Frame::check(src)?;
                }

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
                
                Ok(Frame::Simple(string))
            },
            b'$' => {
                let line = get_line(src)?.to_vec();

                let string = String::from_utf8(line)?;
                
                Ok(Frame::Bulk(string))
            },
            b'*' => {
                let len = get_int(src)?;

                let mut result = Vec::with_capacity(len);

                for _ in 0..len {
                    result.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(result))
            },

            any => {
                println!("WE GOT THIS {}", String::from_utf8(vec![any]).unwrap());
                unimplemented!()
            }
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
            Frame::Array(_) => unimplemented!()
        }
    }
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
        get_line(src)?.to_vec())?
        .parse()
        .map_err(|_| 
            FrameError::Other("Invalid length type".into())
        )
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i+1] == b'\n' && src.get_ref()[i] == b'\r' {
        
            src.set_position((i + 2) as u64);

            let res = &src.get_ref()[start..i];

            return Ok(res)
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

impl From<ParseIntError> for FrameError {
    fn from(value: ParseIntError) -> FrameError {
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple () {
        let input = "OK";

        let check = Frame::Simple(
            input.to_owned()
        );
        
        let input = format!("+{}\r\n", input);
        let input = input.as_bytes();

        let mut cursor = Cursor::new(&input[..]);

        let frame = Frame::parse(&mut cursor).unwrap();

        assert_eq!(check, frame);
    }

    #[test]
    fn test_parse_bulk () {
        // $5\r\nhello\r\n
        assert!(false)
    }

    #[test]
    fn test_parse_empty_bulk () {
        // $0\r\n\r\n
        assert!(false)
    }
    
    #[test]
    fn test_parse_null_bulk () {
        // $-1\r\n
        assert!(false)
    }

    #[test]
    fn test_parse_array () {
        // let input = b"*1\r\n$4\r\nPING\r\n";
        assert!(false)
    }
}