use std::{default::Default, fmt};

use serde::{Deserialize, Serialize};

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use futures_util::stream::{SplitSink, SplitStream};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub enum GameState {
    #[default]
    Preparing, // Preparing the game, accepting
    Lobby,      // Accepts new players
    LobbyReady, // The game can be started
    Playing,    // Playing
    Stopping,   // Closing connections
    Stopped,    // Please destroy this instance
}

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
    },
    Kick {
        player: u32,
    },
    Stop,
    // Data from the user forwarded to the game
    From {
        from: u32,
        data: String,
    },
    // Data from the game, forwarded to the user
    To {
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

/// Version 1
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum GameAction {
    #[default]
    Idle, // Idle
    Start,         // Start the game, from host
    Countdown,     // Sends a countdown request to host, host send Countdown back when done
    Replay,        // Play again (from AfterGame screen)
    ReplayNew,     // Replay with different players (keep the host, remove players)
    Stop,          // Stop the game, from host
    Ready,         // Ready from users
    Timeout,       // Stop because of a Timeout
    RequestText,   // Request the user for a text
    AnnotateImage, // Request the user to anotate an image
    Show,          // Show an image
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameResponse {
    pub id: i32,            // Same ID as the request
    pub action: GameAction, // Action that was requested
    pub text: Vec<String>,  // Inputs
    pub width: u16,         // Width of the following data
    pub data: Vec<u8>,      // Array for large amount of data
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug)]
pub struct GameResponseWithSource {
    pub source: String,
    pub index: u32,
    pub msg: GameResponse,
}

// To the clients & the host
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameRequest {
    pub id: i32,             // Unique identifier
    pub action: GameAction,  // The action to be done
    pub title: String,       // Title of the action
    pub description: String, // Some description of what needs to be done (optional)
    pub size: i32,           // Action specific, typically the number of replies (such as )
    pub width: u16,          // Width of the data (for images)
    pub data: Vec<u8>,       // Data provided with the action
    pub time_s: i32,         // Time available for the action
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Button {
    id: String,
    title: String,
    descr: String,
    text: String,
}

// Another message idea?
#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    data: Vec<u8>,
    width: u32,
    id: String,
}

/// Version 2
#[derive(Serialize, Deserialize, Debug)]
pub enum Msg2Impl {
    /// Host
    Idle, // To
    Countdown {
        // To & from, time = 0 means continue
        time: i32,
    },
    Scores(String), // To
    Start,          // From
    Stop(String),   // From

    /// User
    ButtonReq(Button), // To
    ButtonsReq(Vec<Button>), // To
    ButtonResp {
        // From
        id: String, // Button ID
    },
    TextReq {
        id: String,
        title: String,
        descr: String,
        number: u8,
    },
    TextRes {
        id: String,
        text: Vec<String>,
    },

    ShowImage(Image),     // To
    AnnotateReq(Image),   // To
    AnnotateResp(String), // From
    DrawRequest {
        // To
        id: String,
        title: String,
        descr: String,
    },
    DrawResponse(Image), // From
}

impl fmt::Display for Msg2Impl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg2 {
    cmd: String,
    data: Msg2Impl,
}

impl Msg2 {
    pub fn new(data: Msg2Impl) -> Self {
        Self {
            cmd: data.to_string(),
            data,
        }
    }
}
