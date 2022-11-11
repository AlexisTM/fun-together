use std::net::TcpStream;
use tungstenite::{Error, Message, WebSocket};

pub struct Actor {
    name: String,
    ws: WebSocket<TcpStream>,
}

impl Actor {
    pub fn new(name: String, ws: WebSocket<TcpStream>) -> Self {
        Self { name, ws }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        return self.ws.read_message();
    }
}
