use std::default::Default;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum GameAction {
    #[default]
    Idle,          // Idle
    Start,         // Start the game, from host
    Countdown,     // Sends a countdown request to host, host send Countdown back when done
    Replay,        // Play again (from AfterGame screen)
    ReplayNew,     // Replay with different players (keep the host, remove players)
    Stop,          // Stop the game, from host
    Timeout,       // Stop because of a Timeout
    RequestText,   // Request the user for a text
    AnnotateImage, // Request the user to anotate an image
    Show,          // Show an image
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    Preparing,           // Preparing the game, accepting
    Lobby,               // Accepts new players
    LobbyReady,          // The game can be started
    LobbyReadyCountdown, // The game can be started
    Playing,             // Playing
    AfterGame,           // Shows stats & propose to replay
    Stopping,            // Stopping the game
    Stopped,             // Please destroy this instance
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameResponse {
    pub id: String,         // Same ID as the request
    pub action: GameAction, // Action that was requested
    pub texte: Vec<String>, // Inputs
    pub width: u16,         // Width of the following data
    pub data: Vec<u8>,      // Array for large amount of data
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug)]
pub struct GameResponseWithSource {
    pub source: String,
    pub msg: GameResponse,
}

// To the clients & the host
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameRequest {
    pub id: String,          // Unique identifier
    pub action: GameAction,  // The action to be done
    pub title: String,       // Title of the action
    pub description: String, // Some description of what needs to be done (optional)
    pub size: i32,           // Action specific, typically the number of replies (such as )
    pub width: u16,          // Width of the data (for images)
    pub data: Vec<u8>,       // Data provided with the action
    pub time_s: i32,         // Time available for the action
}
