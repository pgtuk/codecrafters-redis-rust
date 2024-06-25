
use super::Frame;


pub trait ClientCmd {
    // command representation in RESP:
    // A client sends a request to the Redis server as an array of strings.
    // The array frame containing the command and its arguments that the server should execute
    fn to_frame(&self) -> Frame;
}