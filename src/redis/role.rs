use core::fmt;
use std::fmt::Formatter;

#[derive(Clone, PartialEq)]
pub enum Role {
    Master,
    Slave,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let role = match self {
            Role::Master => "master",
            Role::Slave => "slave",
        };

        write!(f, "{}", role)
    }
}