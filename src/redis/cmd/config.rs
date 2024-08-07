use std::fmt;

use anyhow::Result;

use crate::redis::frame::Frame;
use crate::redis::parser::Parser;
use crate::redis::ServerInfo;
use crate::redis::utils::Named;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Config {
    pub subcommand: Subcommand
}

impl Config {
    pub(crate) fn parse_args(parser: &mut Parser) -> Result<Config> {
        let subcommand_as_str = parser.next_string()?.to_uppercase();
        let subcommand = match &subcommand_as_str[..] {
            "GET" => {Subcommand::Get(GetParams::parse(parser)?)},
            _ => unimplemented!()
        };

        Ok(Config{ subcommand })
    }

    pub(crate) fn apply (&self, server_info: &ServerInfo) -> Frame {
        let mut resp = Frame::array();

        match &self.subcommand {
            Subcommand::Get(params) => {
                for param in params {
                    resp.extend(param.to_frame(server_info))
                }
            }
        }

        resp
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Subcommand {
    // None,
    Get(Vec<GetParams>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum GetParams {
    Dir,
    DBfilename
}

impl GetParams {
    fn parse(parser: &mut Parser) -> Result<Vec<GetParams>> {
        let mut params = vec![];
        while let Ok(param) = parser.next_string() {
            match param.to_lowercase().as_str() {
                "dir" => params.push(GetParams::Dir),
                "dbfilename" => params.push(GetParams::DBfilename),
                _ => unimplemented!()
            }
        };

        Ok(params)
    }
    fn to_frame(&self, server_info: &ServerInfo) -> Vec<Frame> {
        let mut result = vec![];
        match self {
            GetParams::Dir => {
                result.push(Frame::Bulk("dir".into()));
                result.push(Frame::Bulk(server_info.dir.clone().into()));
            },
            GetParams::DBfilename => {
                result.push(Frame::Bulk("dbfilename".into()));
                result.push(Frame::Bulk(server_info.db_file.clone().into()));
            }
        }

        result
    }
}

impl Named for Config {
    const NAME: &'static str = "CONFIG";
}

impl fmt::Display for Subcommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Subcommand::Get(_) => write!(f, "GET"),
            // ReplconfParam::Capa => write!(f, "capa"),
            // ReplconfParam::Getack => write!(f, "GETACK")
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::redis::cmd::Command;
    use crate::redis::tests::make_frame;

    use super::*;

    #[test]
    fn test_cmd_config_get_from_frame() {
        let input = b"*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$3\r\ndir\r\n";
        let frame = make_frame(input);

        let cmd = Command::from_frame(&frame).unwrap();

        let expected = Command::Config(
            Config {
                subcommand: Subcommand::Get(vec![GetParams::Dir])
            }
        );

        assert_eq!(
            cmd,
            expected,
        )
    }
}