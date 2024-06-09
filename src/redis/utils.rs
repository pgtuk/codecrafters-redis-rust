use std::fmt;

#[derive(Clone, Debug)]
pub struct Addr {
    pub host: String,
    pub port: String,
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl Default for Addr {
    fn default() -> Self {
        Addr {
            host: String::from("127.0.0.1"),
            port: String::from("6379"),
        }
    }
}