use anyhow::Result;

use super::{
    connection::Connection,
    frame::Frame,
    parser::Parser,
};

mod ping;
use ping::Ping;

mod echo;
use echo::Echo;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        // all redis commands come in form of RESP arrays
        let mut parser = Parser::new(frame)?;
  
        let command_name = parser.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_args(&mut parser)?),
            "echo" => Command::Echo(Echo::parse_args(&mut parser)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<()> {
        let response_frame = match self {
            Command::Ping(cmd) => {cmd.to_response()},
            Command::Echo(cmd) => {cmd.to_response()},
            // _ => unimplemented!()
        };

        conn.write_frame(&response_frame).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
