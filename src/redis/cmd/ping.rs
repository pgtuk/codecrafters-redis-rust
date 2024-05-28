use crate::{
    redis::{
        Connection,
        frame::Frame,
        parser::{
            Parser, 
            ParserError,
        },
    },
    Result,
};

#[derive(Debug)]
pub struct Ping {
    msg: Option<String>,
}

impl Ping {
    pub fn new(msg: Option<String>) -> Ping {
        Ping { msg }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Ping> {
        match parser.next_string() {
            Ok(args) => Ok(Ping::new(Some(args))),
            Err(ParserError::EndOfStream) => Ok(Ping::new(None)),
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn apply (self, conn: &mut Connection) -> Result<()> {

        let resp = match self.msg {
            None => Frame::Simple("PONG".to_owned()),
            Some(_msg) => unimplemented!(),
        };

        conn.write_frame(&resp).await?;

        Ok(())
    }
}