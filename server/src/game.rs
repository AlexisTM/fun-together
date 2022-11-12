use crate::actor::Actor;
use crate::comm::{GameRequest, GameResponse, GameState};

use std::sync::RwLock;

use tungstenite::Message;

pub struct Game {
    state: GameState,
    name: String,
    host: RwLock<Actor>,
    players: RwLock<Vec<Actor>>,
    min_players: u16,
    max_players: u16,
}

impl Game {
    pub fn new(name: String, host: Actor, min_players: u16, max_players: u16) -> Self {
        Self {
            state: GameState::Preparing,
            name,
            host: RwLock::new(host),
            players: RwLock::new(Vec::new()),
            min_players,
            max_players,
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

    pub fn state(self) -> GameState {
        self.state
    }

    pub fn update(self) {
        if self.state == GameState::LobbyReady {}

        match self.state {
            GameState::Preparing => self.update_preparing(), // Preparing the game, accepting
            GameState::Lobby => self.update_lobby(),         // Accepts new players
            GameState::LobbyReady => self.update_lobby(),    // The game can be started
            GameState::Playing => self.update_playing(),     // Playing
            GameState::AfterGame => self.update_aftergame(), // Shows stats & propose to replay
        }
    }

    pub fn update_preparing(self) {}
    pub fn update_lobby(self) {}
    pub fn update_playing(self) {}
    pub fn update_aftergame(self) {}
}
