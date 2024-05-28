use super::{
    connection::Connection,
    frame::Frame,
    parser::Parser,
    ProtocolError,
};

mod ping;
use ping::Ping;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    // Echo,
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command, ProtocolError> {
        // all redis commands come in form of RESP arrays
        let mut parser = Parser::new(frame)?;
  
        let command_name = parser.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_args(&mut parser)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<(), ProtocolError> {
        match self {
            Command::Ping(cmd) => {cmd.apply(conn).await}
        }
    }
}

#[cfg(test)]
mod tests;
