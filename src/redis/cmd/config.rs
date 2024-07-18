use std::fmt;

use anyhow::Result;

use crate::redis::frame::Frame;
use crate::redis::parser::Parser;
use crate::redis::utils::Named;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Config {
    subcommand: Subcommand
}

impl Config {
    pub(crate) fn parse_args(parser: &mut Parser) -> Result<Config> {
        let subcommand_as_str = parser.next_string()?.to_uppercase();
        let subcommand = match &subcommand_as_str[..] {
            "GET" => {Subcommand::GET(GetParams::parse(parser)?)},
            _ => unimplemented!()
        };

        Ok(Config{ subcommand })
    }

    pub(crate) fn apply (&self) -> Frame {
        Frame::Null
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Subcommand {
    // None,
    GET(Vec<GetParams>),
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
}

impl Named for Config {
    const NAME: &'static str = "CONFIG";
}

impl fmt::Display for Subcommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Subcommand::GET(_) => write!(f, "GET"),
            // ReplconfParam::Capa => write!(f, "capa"),
            // ReplconfParam::Getack => write!(f, "GETACK")
        }
    }
}