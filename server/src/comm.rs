use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Action {
    Idle,          // Idle
    Start,         // Start the game, from host
    Stop,          // Stop the game, from host
    Timeout,       // Stop because of a Timeout
    RequestText,   // Request the user for a text
    AnnotateImage, // Request the user to anotate an image
    Show,          // Show an image
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum GameState {
    Preparing,  // Preparing the game, accepting
    Lobby,      // Accepts new players
    LobbyReady, // The game can be started
    Playing,    // Playing
    AfterGame,  // Shows stats & propose to replay
    Stopping,   // Stopping the game
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug)]
pub struct GameResponse {
    id: String,         // Same ID as the request
    action: Action,     // Action that was requested
    texte: Vec<String>, // Inputs
    width: u16,         // Width of the following data
    data: Vec<u8>,      // Array for large amount of data
}

// From the clients & the host
#[derive(Serialize, Deserialize, Debug)]
pub struct GameResponseWithSource {
    pub source: String,
    pub msg: GameResponse,
}

// To the clients & the host
#[derive(Serialize, Deserialize, Debug)]
pub struct GameRequest {
    id: String,          // Unique identifier
    action: Action,      // The action to be done
    title: String,       // Title of the action
    description: String, // Some description of what needs to be done (optional)
    size: i32,           // Action specific, typically the number of replies (such as )
    width: u16,          // Width of the data (for images)
    data: Vec<u8>,       // Data provided with the action
    time_s: i32,         // Time available for the action
}
