use anyhow::Result;

use super::{
    connection::Connection, 
    db::Db, 
    frame::Frame, 
    parser::Parser,
    ServerInfo,
};

pub mod client_cmd;

mod echo;
use echo::Echo;
mod get;
use get::Get;
mod info;
use info::Info;
mod ping;
pub use ping::Ping;
mod set;
use set::Set;
pub mod replconf;
use replconf::Replconf;


#[derive(Debug, PartialEq)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Info(Info),
    Replconf(Replconf)
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        // all redis commands come in form of RESP arrays
        let mut parser = Parser::new(frame)?;
  
        let command_name = parser.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_args(&mut parser)?),
            "echo" => Command::Echo(Echo::parse_args(&mut parser)?),
            "set" => Command::Set(Set::parse_args(&mut parser)?),
            "get" => Command::Get(Get::parse_args(&mut parser)?),
            "info" => Command::Info(Info::parse_args()?),
            // TODO: fix hardcoded
            "replconf" => Command::Replconf(Replconf { param: replconf::ReplconfParam::ListeningPort, arg: "args".to_string() } ),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, conn: &mut Connection, db: &mut Db, info: &ServerInfo) -> Result<()> {
        // returns result of calling the command on server side
        let response_frame = match self {
            Command::Ping(cmd) => {cmd.apply()},
            Command::Echo(cmd) => {cmd.apply()},
            Command::Set(cmd) => {cmd.apply(db)},
            Command::Get(cmd) => {cmd.apply(db)},
            Command::Info(cmd) => {cmd.apply(info)},
            Command::Replconf(cmd) => {cmd.apply()},
            // _ => unimplemented!()
        };

        conn.write_frame(&response_frame).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
