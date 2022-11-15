use futures_util::{StreamExt, stream::{SplitStream, SplitSink}, TryStreamExt};
use futures_util::{future};
use serde::Serialize;
use std::borrow::Cow;

use tokio::{
    net::{TcpStream, tcp::{WriteHalf, ReadHalf}}, io::{AsyncReadExt, AsyncWriteExt}
};
use tokio_tungstenite::{
    tungstenite::protocol::{frame::coding::CloseCode, CloseFrame},
    tungstenite::{Error, Message},
    WebSocketStream,
};

use crate::comm::Command;

pub struct Actor<'a> {
    id: usize,
   // ws: WebSocketStream<TcpStream>,
    ws_tx: WriteHalf<'a>,
    ws_rx: ReadHalf<'a>,
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

impl Actor<'_> {
    pub fn new(id: usize, ws: WebSocketStream<TcpStream>) -> Self {
        let (rx, tx) = ws.get_mut().split();
        Self {
            id,
            // ws,
            ws_tx: tx,
            ws_rx: rx,
        }
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        // self.ws_rx.filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        return Err(Error::AlreadyClosed);
    }

    // Read as text
    pub fn read_command(&mut self) -> Option<Command> {
        /*
        let msg = self.ws.next().await.unwrap();
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
        */
        None
    }

    pub fn read_text(&mut self) -> Option<String> {
        /*
        let msg = self.ws.read_message();
        if msg.is_ok() {
            let result = msg.unwrap_or_else(|_| Message::Ping(vec![]));
            match result {
                Message::Text(x) => Some(x),
                _ => None,
            }
        } else {
            None
        }*/
        None
    }

    pub fn send_request<T: Serialize>(&mut self, data: &T) {
        /*
        if let Ok(msg) = serde_json::to_string(data) {
            self.ws.send(Message::Text(msg)).await.unwrap();
        } else {
            // Error serializing
        }*/
    }

    pub fn write(&mut self, data: String) {
        // self.ws.write_message(Message::Text(data)).unwrap_or(());
    }

    pub fn write_message(&mut self, data: Message) {
        // self.ws.write_message(data).unwrap_or(());
    }

    pub fn get_id(&mut self) -> usize {
        self.id
    }

    pub fn disconnect(&mut self, code: CloseCode) {
        /*
        let reason = Cow::Borrowed("Bye! <3");
        self.ws
            .close(Some(CloseFrame { code, reason }))
            .unwrap_or(());
        */
    }
}
