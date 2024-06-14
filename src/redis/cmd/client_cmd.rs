
use super::Frame;


pub trait ClientCmd {
    fn to_frame(&self) -> Frame;
}