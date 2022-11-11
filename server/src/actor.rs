use std::net::TcpStream;
use tungstenite::{Error, Message, WebSocket};

pub struct Actor {
    name: String,
    ws: WebSocket<TcpStream>,
}

pub fn debug_msg(msg: Message) {
    match msg {
        Message::Text(x) => println!("Text: {}", x),
        Message::Binary(x) => println!("Binary: {:?}", x),
        Message::Ping(x) => println!("Ping: {:?}", x),
        Message::Pong(x) => println!("Pong: {:?}", x),
        Message::Close(x) => println!("Close: {:?}", x),
        Message::Frame(x) => println!("Frame: {}", x),
    }
}

impl Actor {
    pub fn new(name: String, ws: WebSocket<TcpStream>) -> Self {
        Self { name, ws: ws }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        return self.ws.read_message();
    }

    // Read as text
    pub fn read_text(&mut self) -> Option<String> {
        let msg = self.ws.read_message();
        if msg.is_ok() {
            let result = msg.unwrap();
            match result {
                Message::Text(x) => Some(x),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn write(&mut self, data: String) {
        self.ws.write_message(Message::Text(data)).unwrap();
    }

    pub fn write_message(&mut self, data: Message) {
        self.ws.write_message(data).unwrap();
    }
}
