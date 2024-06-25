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

pub trait Named {
    const NAME: &'static str;

    fn name(&self) -> String {
        Self::NAME.into()
    }
}

pub fn int_as_bytes(i: &usize) -> Vec<u8> {
    let mut buff = Vec::new();

    for c in i.to_string().chars() {
        buff.push(c as u8);
    }

    buff
}

pub fn add_cr(buff: &mut Vec<u8>) {
    buff.extend([b'\r', b'\n']);
}