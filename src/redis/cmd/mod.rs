use anyhow::Result;

use config::Config as ConfigCmd;
use echo::Echo;
use get::Get;
use info::Info;
pub(crate) use ping::Ping;
pub(crate) use psync::Psync;
use replconf::Replconf;
use set::Set;
pub(crate) use wait::Wait;

use super::{
    frame::Frame,
    parser::Parser,
};

pub mod get;
pub mod replconf;

mod config;
mod echo;
mod info;
mod ping;
mod set;
mod psync;
mod wait;


#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Info(Info),
    Replconf(Replconf),
    Psync(Psync),
    Wait(Wait),
    Config(ConfigCmd)
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
            "wait" => Command::Wait(Wait::parse_args(&mut parser)?),
            "config" => Command::Config(ConfigCmd::parse_args(&mut parser)?),
            _ => unimplemented!(),
        };

        Ok(command)
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
