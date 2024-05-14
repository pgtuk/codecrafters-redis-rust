use crate::Result;

use super::connection::Connection;
use super::frame::Frame;

mod ping;
use ping::Ping;


pub enum Command {
    Ping(Ping),
    // Echo,
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        let (command_name, args) =  match frame {
            Frame::Bulk(s) => {parse_line(&s)},
            Frame::Simple(s) => {parse_line(&s)},
        };

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_args(args)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<()> {
        match self {
            Command::Ping(cmd) => {cmd.apply(conn).await}
        }
    }
}

fn parse_line(line: &String) -> (String, Option<String>) {
    let line = line.to_lowercase();
    match line.split_once(' ') {
        Some((command, args)) => (
            command.to_owned(), 
            Some(args.to_owned()),
        ),
        None => (line.to_owned(), None)
    }
}