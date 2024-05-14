use super::Result;

use crate::redis::frame::Frame;

use super::Connection;

pub struct Ping {
    msg: Option<String>,
}

impl Ping {
    pub fn new(msg: Option<String>) -> Ping {
        Ping { msg }
    }

    pub fn parse_args(args: Option<String>) -> Result<Ping> {
        match args {
            Some(args) => Ok(Ping::new(Some(args))),
            None => Ok(Ping::new(None)),
        }
    }
    
    pub async fn apply (self, conn: &mut Connection) -> Result<()> {

        let resp = match self.msg {
            None => Frame::Simple("PONG".to_owned()),
            Some(msg) => Frame::Bulk(msg), 
        };

        conn.write_frame(&resp).await?;

        Ok(())
    }
}