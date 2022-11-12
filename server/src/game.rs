use crate::actor::Actor;
use crate::comm::{GameRequest, GameResponse, GameResponseWithSource, GameState};

use std::sync::RwLock;

use tungstenite::Message;

pub struct Game {
    state: GameState,
    name: String,
    host: RwLock<Actor>,
    players: RwLock<Vec<Actor>>,
    min_players: usize,
    max_players: usize,
}

impl Game {
    pub fn new(name: String, host: Actor, min_players: usize, max_players: usize) -> Self {
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

    pub fn state(self) -> GameState {
        self.state
    }

    pub fn update(&mut self) {
        let mut players = self.players.write().unwrap();
        let mut host = self.host.write().unwrap();

        let host_message = host.read_response();
        let mut messages: Vec<GameResponseWithSource> = Vec::new();

        if let Some(msg) = host_message {

        }
        players.iter_mut().for_each(|player| {
            if let Some(result) = player.read_response() {
                messages.push(GameResponseWithSource {
                    msg: result,
                    source: player.get_name().to_string(),
                });
            }
        });

        match self.state {
            GameState::Preparing => {
                host.set_ready();
                if host.ready() {
                    self.state = GameState::Lobby;
                }
            } // Preparing the game, accepting
            GameState::Lobby => {
                let enough_players =
                    players.len() >= self.min_players && players.len() <= self.max_players;
                let everybody_ready = players.iter().all(|actor| actor.ready());
                if self.state == GameState::Lobby {
                    if enough_players && everybody_ready {
                        self.state = GameState::LobbyReady;
                    }
                }
            } // Accepts new players
            GameState::LobbyReady => {
                let enough_players =
                    players.len() >= self.min_players && players.len() <= self.max_players;
                let everybody_ready = players.iter().all(|actor| actor.ready());
                if !enough_players || !everybody_ready {
                    self.state = GameState::Lobby;
                }
            } // The game can be started
            GameState::Playing => {}   // Playing
            GameState::AfterGame => {} // Shows stats & propose to replay
            GameState::Stopping => {}  // Shows stats & propose to replay
        }
    }
}
