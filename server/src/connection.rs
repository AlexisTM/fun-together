use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use futures_util::stream::{SplitSink, SplitStream};

#[derive(Debug)]
pub struct ToPlayer {
    pub id: u32,
    pub sender: SplitSink<WebSocketStream<TcpStream>, Message>,
}
impl ToPlayer {
    pub fn new(id: u32, sender: SplitSink<WebSocketStream<TcpStream>, Message>) -> Self {
        Self { id: id, sender }
    }
}

#[derive(Debug)]
pub struct FromPlayer {
    pub id: u32,
    pub receiver: SplitStream<WebSocketStream<TcpStream>>,
}
impl FromPlayer {
    pub fn new(id: u32, receiver: SplitStream<WebSocketStream<TcpStream>>) -> Self {
        Self { id: id, receiver }
    }
}
