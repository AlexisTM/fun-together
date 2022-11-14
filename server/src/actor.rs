use serde::Serialize;
use std::{borrow::Cow, net::TcpStream};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Error, Message, WebSocket,
};

use crate::comm::{Command};

pub struct Actor {
    id: u32,
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
    pub fn new(id: u32, ws: WebSocket<TcpStream>) -> Self {
        Self {
            id,
            ws,
        }
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        self.ws.read_message()
    }

    // Read as text
    pub fn read_command(&mut self) -> Option<Command> {
        let msg = self.ws.read_message();
        let string_msg = if msg.is_ok() {
            let result = msg.unwrap_or_else(|_| Message::Ping(vec![]));
            match result {
                Message::Text(x) => Some(x),
                _ => None,
            }
        } else {
            None
        };

        if let Some(val) = string_msg {
            let json: Result<Command, _> = serde_json::from_str(val.as_str());
            match json {
                Ok(valid_json) => Some(valid_json),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn read_text(&mut self) -> Option<String> {
        let msg = self.ws.read_message();
        if msg.is_ok() {
            let result = msg.unwrap_or_else(|_| Message::Ping(vec![]));
            match result {
                Message::Text(x) => Some(x),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn send_request<T: Serialize>(&mut self, data: &T) {
        if let Ok(msg) = serde_json::to_string(data) {
            self.ws.write_message(Message::Text(msg)).unwrap_or(());
        } else {
            // Error serilizing
        }
    }

    pub fn write(&mut self, data: String) {
        self.ws.write_message(Message::Text(data)).unwrap_or(());
    }

    pub fn write_message(&mut self, data: Message) {
        self.ws.write_message(data).unwrap_or(());
    }

    pub fn get_id(&mut self) -> u32 {
        self.id
    }

    pub fn disconnect(&mut self, code: CloseCode) {
        let reason = Cow::Borrowed("Bye! <3");
        self.ws
            .close(Some(CloseFrame { code, reason }))
            .unwrap_or(());
    }
}
