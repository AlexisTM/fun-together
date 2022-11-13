use crate::actor::Actor;
use crate::comm::{Command, GameRequest, GameResponseWithSource, GameState};

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{RwLock, RwLockWriteGuard};

use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::WebSocket;

pub struct Game {
    last_id_given: u32,
    state: GameState,
    last_state: GameState,
    host: RwLock<Actor>,
    players: RwLock<HashMap<u32, Actor>>,
    min_players: usize,
    max_players: usize,
}

impl Game {
    pub fn new(host: Actor) -> Self {
        Self {
            last_id_given: 0,
            last_state: GameState::Preparing,
            state: GameState::Preparing,
            host: RwLock::new(host),
            players: RwLock::new(HashMap::new()),
            min_players: 0,
            max_players: 0,
        }
    }

    pub fn get_host(&mut self) -> &mut RwLock<Actor> {
        &mut self.host
    }

    pub fn add(&mut self, mut player_ws: WebSocket<TcpStream>) -> bool {
        let mut players = self.players.write().unwrap();
        self.last_id_given += 1;
        let mut player = Actor::new(self.last_id_given, player_ws);
        if players.len() < self.max_players
            && (self.state == GameState::Lobby || self.state == GameState::LobbyReady)
        {
            players.insert(self.last_id_given, player);
            return true;
        }
        player.disconnect(CloseCode::Error);

        false
    }

    pub fn state(self) -> GameState {
        self.state
    }

    pub fn update(&mut self) -> bool {
        let curr_state = self.state.clone();
        let mut players = self.players.write().unwrap();
        let mut host = self.host.write().unwrap();

        let host_message = host.read_command();
        let mut messages: Vec<GameResponseWithSource> = Vec::new();

        if let Some(msg) = &host_message {
            if matches!(msg, Command::Stop()) {
                self.state = GameState::Stopping;
            }
        }

        if self.state != self.last_state {
            println!("{:?}", self.state);
        }

        players.iter_mut().for_each(|(id, player)| {
            /// FORWARD DATA
            if let Some(result) = player.read_text() {}
        });

        match self.state {
            GameState::Preparing => {
                if let Some(msg) = &host_message {
                    if matches!(
                        msg,
                        Command::Prepare {
                            min_players,
                            max_players
                        }
                    ) {
                        self.state = GameState::Playing;
                    }
                }

                host.send_request(&GameRequest::default());
                host.send_request(&Command::State {
                    players: vec![1, 2, 3],
                    state: self.state.clone(),
                });
                host.send_request(&Command::Prepare {
                    min_players: 2,
                    max_players: 8,
                });
                host.send_request(&Command::Stop());
                host.send_request(&Command::PrepareReply {
                    key: "hey".to_string(),
                });
                host.send_request(&Command::From {
                    from: 1,
                    data: "".to_string(),
                });
                host.send_request(&Command::To {
                    to: vec![1, 2],
                    data: "".to_string(),
                });
            } // Resettting the game data, & accepting
            GameState::Lobby => {
                let enough_players =
                    players.len() >= self.min_players && players.len() <= self.max_players;

                if enough_players {
                    self.state = GameState::LobbyReady;
                }
            } // Accepts new players
            GameState::LobbyReady => {
                let enough_players =
                    players.len() >= self.min_players && players.len() <= self.max_players;

                if !enough_players {
                    self.state = GameState::Lobby;
                } else {
                    if let Some(msg) = &host_message {
                        if matches!(msg, Start) {
                            self.state = GameState::Playing;
                        }
                    }
                }
            } // The game can be started
            GameState::Playing => {
                // Leaves on stop message from host, executed before the SM
            }
            GameState::Stopping => {
                players.iter_mut().for_each(|(u32, player)| {
                    player.disconnect(CloseCode::Away);
                });
                host.disconnect(CloseCode::Away);
            } // Cleanup of the game and destruction of all sessions
            GameState::Stopped => {
                // Nothing to do
                return false;
            } // Cleanup of the game and destruction of all sessions
        }
        self.last_state = curr_state;
        true
    }
}
