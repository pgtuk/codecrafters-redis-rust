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
mod psync;
pub use psync::Psync;


#[derive(Debug, PartialEq)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Info(Info),
    Replconf(Replconf),
    Psync(Psync),
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
            "replconf" => Command::Replconf(Replconf::parse_args(&mut parser)?),
            "psync" => Command::Psync(Psync::parse_args(&mut parser)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, conn: &mut Connection, db: &mut Db, info: &ServerInfo) -> Result<()> {
        // returns result of calling the command on server side
        match self {
            Command::Ping(cmd) => {cmd.apply(conn).await?},
            Command::Echo(cmd) => {cmd.apply(conn).await?},
            Command::Set(cmd) => {cmd.apply(conn, db).await?},
            Command::Get(cmd) => {cmd.apply(conn, db).await?},
            Command::Info(cmd) => {cmd.apply(conn, info).await?},
            Command::Replconf(cmd) => {cmd.apply(conn).await?},
            Command::Psync(cmd) => {cmd.apply(conn, info).await?},
            // _ => unimplemented!()
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests;
