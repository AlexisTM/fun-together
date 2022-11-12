use std::{borrow::Cow, net::TcpStream};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Error, Message, WebSocket,
};

use crate::comm::{GameRequest, GameResponse};

pub struct Actor {
    name: String,
    ready: bool,
    score: i32,
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
        Self {
            name,
            ready: false,
            score: 0,
            ws,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        return self.ws.read_message();
    }

    // Read as text
    pub fn read_response(&mut self) -> Option<GameResponse> {
        let msg = self.ws.read_message();
        let string_msg = if msg.is_ok() {
            let result = msg.unwrap_or(Message::Text("".to_string()));
            match result {
                Message::Text(x) => Some(x),
                _ => None,
            }
        } else {
            None
        };

        if let Some(val) = string_msg {
            let json: Result<GameResponse, _> = serde_json::from_str(val.as_str());
            match json {
                Ok(valid_json) => return Some(valid_json),
                Err(_) => return None,
            }
        } else {
            None
        }
    }

    pub fn write(&mut self, data: String) {
        self.ws.write_message(Message::Text(data)).unwrap_or(());
    }

    pub fn write_message(&mut self, data: Message) {
        self.ws.write_message(data).unwrap_or(());
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn set_ready(&mut self) {
        self.ready = true;
    }

    pub fn set_score(&mut self, score: i32) {
        self.score = score;
    }

    pub fn add_score(&mut self, score: i32) {
        self.score += score;
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn disconnect(&mut self, code: CloseCode) {
        let reason = Cow::Borrowed("Bye! <3");
        self.ws
            .close(Some(CloseFrame {
                code: code,
                reason: reason,
            }))
            .unwrap_or(());
    }
}
