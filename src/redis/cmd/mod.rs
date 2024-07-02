use anyhow::Result;

use echo::Echo;
use get::Get;
use info::Info;
pub use ping::Ping;
pub use psync::Psync;
use replconf::Replconf;
use set::Set;

use super::{
    connection::Connection,
    db::Db,
    frame::Frame,
    parser::Parser,
    ServerInfo,
};

mod echo;
pub mod get;
mod info;
mod ping;
mod set;
pub mod replconf;
mod psync;

#[derive(Debug, PartialEq, Clone)]
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
    pub fn from_frame(frame: &Frame) -> Result<Command> {
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

    pub async fn apply(&self, conn: &mut Connection, db: &mut Db, info: &ServerInfo) -> Result<()> {
        // returns result of calling the command on server side
        let response = match self {
            Command::Ping(cmd) => { cmd.apply() }
            Command::Echo(cmd) => { cmd.apply() }
            Command::Set(cmd) => { cmd.apply(db) }
            Command::Get(cmd) => { cmd.apply(db) }
            Command::Info(cmd) => { cmd.apply(info) }
            Command::Replconf(cmd) => { cmd.apply() }
            Command::Psync(cmd) => { cmd.apply(info) }
            // _ => unimplemented!()
        };

        if !conn.is_repl_conn {
            conn.write_frame(&response).await?;
            if let Command::Psync(_) = self {
                conn.write_rdb(&db.build_rdb_frame()).await?
            }
        }

        Ok(())
    }
}

pub trait ClientCmd {
    // command representation in RESP:
    // A client sends a request to the Redis server as an array of strings.
    // The array frame containing the command and its arguments that the server should execute
    fn to_frame(&self) -> Frame;
}

#[cfg(test)]
mod tests;
