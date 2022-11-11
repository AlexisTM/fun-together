use crate::actor::Actor;

use std::sync::RwLock;

use tungstenite::Message;

pub enum State {
    Preparing,  // Preparing the game, accepting
    Lobby,
    LobbyReady, // The game can be started
    Playing,
    AfterGame,
}

pub struct Game {
    state: State,
    name: String,
    host: RwLock<Actor>,
    players: RwLock<Vec<Actor>>,
}

impl Game {
    pub fn new(name: String, host: Actor) -> Self {
        Self {
            state: State::Preparing,
            name,
            host: RwLock::new(host),
            players: RwLock::new(Vec::new()),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_host(&mut self) -> &mut RwLock<Actor> {
        &mut self.host
    }

    pub fn add(&mut self, player: Actor) {
        self.players.write().unwrap().push(player);
    }

    pub fn handle_actors(&self) {
        let mut players = self.players.write().unwrap();
        for player in players.iter_mut() {
            let msg = player.read();

            if msg.is_ok() {
                let result = msg.unwrap();
                match result {
                    Message::Text(x) => println!("Text from {}: {}", player.get_name(), x),
                    Message::Binary(x) => println!("Binary from {}: {:?}", player.get_name(), x),
                    Message::Ping(x) => println!("Ping from {}: {:?}", player.get_name(), x),
                    Message::Pong(x) => println!("Pong from {}: {:?}", player.get_name(), x),
                    Message::Close(x) => println!("Close from {}: {:?}", player.get_name(), x),
                    Message::Frame(x) => println!("Frame from {}: {}", player.get_name(), x),
                }
            }
        }
    }

    pub fn handle_host(&mut self) {
        let msg = self.host.write().unwrap().read();
        if msg.is_ok() {
            let result = msg.unwrap();
            match result {
                Message::Text(x) => println!("Text from host {}: {}", self.get_name(), x),
                Message::Binary(x) => println!("Binary from host {}: {:?}", self.get_name(), x),
                Message::Ping(x) => println!("Ping from host {}: {:?}", self.get_name(), x),
                Message::Pong(x) => println!("Pong from host {}: {:?}", self.get_name(), x),
                Message::Close(x) => println!("Close from host {}: {:?}", self.get_name(), x),
                Message::Frame(x) => println!("Frame from host {}: {}", self.get_name(), x),
            }
        }
    }

    pub fn state(self) -> State {
        self.state
    }
}
