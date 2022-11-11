use serde::{Deserialize, Serialize};
use serde_cbor;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    RequestText,
    AnnotateImage,
    Show,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    id: String,         // Same ID as the request
    action: Action,     // Action that was requested
    texte: Vec<String>, // Inputs
    width: u16,         // Width of the following data
    data: Vec<u8>,      // Array for large amount of data
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    id: String,          // Unique identifier
    title: String,       // Title of the action
    description: String, // Some description of what needs to be done (optional)
    action: Action,      // The action to be done
    size: i32,           // Action specific, typically the number of replies (such as )
    width: u16,          // Width of the data (for images)
    data: Vec<u8>,       // Data provided with the action
    time_s: i32,         // Time available for the action
}
