use serde::{Deserialize, Serialize};

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use futures_util::stream::{SplitSink, SplitStream};

/// Version 3
/// The game has full control of the comms
#[derive(Debug)]
pub struct PlayerSink {
    pub id: u32,
    pub sink: SplitSink<WebSocketStream<TcpStream>, Message>,
}
impl PlayerSink {
    pub fn new(id: u32, sink: SplitSink<WebSocketStream<TcpStream>, Message>) -> Self {
        Self { id, sink }
    }
}

#[derive(Debug)]
pub struct PlayerStream {
    pub id: u32,
    pub stream: SplitStream<WebSocketStream<TcpStream>>,
}

impl PlayerStream {
    pub fn new(id: u32, stream: SplitStream<WebSocketStream<TcpStream>>) -> Self {
        Self { id, stream }
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub ws: WebSocketStream<TcpStream>,
}
impl Player {
    pub fn new(id: u32, ws: WebSocketStream<TcpStream>) -> Self {
        Self { id, ws }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "cmd")]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Prepare {
        max_players: u32,
    }, // Prepares a game
    PrepareReply {
        key: String, // The game key
    },
    Start, // Prevent players to join from this point on
    State {
        players: Vec<u32>,
        max_players: u32,
        accept_conns: bool,
    },
    Kick {
        player: u32,
    },
    Stop,
    // Data from the user forwarded to the game
    From {
        from: u32,
        data: Vec<u8>,
    },
    FromStr {
        from: u32,
        data: String,
    },
    // Data from the game, forwarded to the user
    To {
        to: Vec<u32>,
        data: Vec<u8>,
    },
    ToStr {
        to: Vec<u32>,
        data: String,
    },
    Error {
        reason: String,
    },
}

#[derive(Debug)]
pub enum HostComm {
    Join(PlayerSink),
    Leave(u32),
    Command(Command),
}
