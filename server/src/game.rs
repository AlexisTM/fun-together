use crate::actor::Actor;
use crate::comm::{GameAction, GameRequest, GameResponse, GameResponseWithSource, GameState};

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

    pub fn add(&mut self, player: Actor) -> bool {
        let players =  self.players.write().unwrap();
        if players.len() < self.max_players {
            if self.state == GameState::Lobby || self.state == GameState::LobbyReady {
                players.push(player);
                return true;
            }
        }
        return false;
    }

    pub fn state(self) -> GameState {
        self.state
    }

    pub fn update(&mut self) -> bool {
        let mut players = self.players.write().unwrap();
        let mut host = self.host.write().unwrap();

        let host_message = host.read_response();
        let mut messages: Vec<GameResponseWithSource> = Vec::new();

        if let Some(msg) = host_message {
            if msg.action == GameAction::Stop {
                self.state = GameState::Stopping;
            }
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

                if let Some(msg) = host_message {
                    if msg.action == GameAction::Start {
                        self.state = GameState::LobbyReadyCountdown;
                    }
                }
            } // The game can be started
            GameState::LobbyReadyCountdown => {
                if let Some(msg) = host_message {
                    if msg.action == GameAction::Countdown {
                        self.state = GameState::Playing;
                    }
                }
            } // Playing
            GameState::Playing => {
                // Logic!!
            }
            GameState::AfterGame => {
                if let Some(msg) = host_message {
                    if msg.action == GameAction::Replay {
                        self.state = GameState::Playing;
                    }
                }
            } // Shows stats & propose to replay
            GameState::Stopping => {
                // Cleanup of the game and destruction of all sessions
            } // Cleanup of the game and destruction of all sessions
            GameState::Stopped => {
                // Nothing to do
                return false;
            } // Cleanup of the game and destruction of all sessions
        }

        return true;
    }
}
