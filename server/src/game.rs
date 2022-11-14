use crate::actor::Actor;
use crate::comm::{Command, GameRequest, GameState};


use std::collections::HashMap;
use std::net::TcpStream;
use parking_lot::{RwLock};

use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::WebSocket;

pub struct Game {
    state: GameState,
    host: Actor,
    players: RwLock<HashMap<usize, Actor>>,
    min_players: usize,
    max_players: usize,
    last_id_given: usize,
}

impl Game {
    pub fn new(mut host: Actor) -> Self {
        host.send_request(&GameRequest::default());
        host.send_request(&Command::State {
            players: vec![1, 2, 3],
            state: GameState::Preparing,
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

        Self {
            last_id_given: 0,
            state: GameState::Preparing,
            host,
            players: RwLock::new(HashMap::new()),
            min_players: 0,
            max_players: 0,
        }
    }

    pub fn add(&mut self, player_ws: WebSocket<TcpStream>) -> bool {
        let mut players = self.players.write();

        self.last_id_given += 1;
        let mut player = Actor::new(self.last_id_given, player_ws);
        if players.len() < self.max_players
            && (self.state == GameState::Lobby || self.state == GameState::LobbyReady)
        {
            players.insert(self.last_id_given, player);
            return true;
        }
        player.disconnect(CloseCode::Error);

        let msg = Command::State {
            players: players.keys().cloned().collect(),
            state: self.state.clone(),
        };
        self.host.send_request(&msg);
        false
    }

    pub fn update(&mut self) -> bool {
        let mut players = self.players.write();

        let curr_state = self.state.clone();

        match self.state {
            GameState::Preparing => {} // Resettting the game data, & accepting
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
                }
            } // The game can be started
            GameState::Playing => {
                // Leaves on stop message from host, executed before the SM
            }
            GameState::Stopping => {
                players.iter_mut().for_each(|(_u32, player)| {
                    player.disconnect(CloseCode::Away);
                });
                self.host.disconnect(CloseCode::Away);
            } // Cleanup of the game and destruction of all sessions
            GameState::Stopped => {
                // Nothing to do
                return false;
            } // Cleanup of the game and destruction of all sessions
        }

        players.iter_mut().for_each(|(id, player)| {
            if let Some(result) = player.read_text() {
                self.host.send_request(&&Command::From {
                    from: *id,
                    data: result,
                });
            }
        });

        while let Some(host_cmd) = self.host.read_command() {
            println!("{:?}", host_cmd);
            match host_cmd {
                Command::Prepare {
                    min_players,
                    max_players,
                } => {
                    if self.state == GameState::Preparing {
                        self.min_players = min_players;
                        self.max_players = max_players;
                        self.state = GameState::Lobby;
                        self.host.send_request(&Command::PrepareReply {
                            key: "HEYX".to_string(),
                        });
                    } else {
                        self.host.send_request(&Command::Error {
                            reason: "The game is not in Preparing state.".to_string(),
                        });
                    }
                }
                Command::Start() => {
                    if self.state == GameState::LobbyReady {
                        self.state = GameState::Playing;
                    }
                }
                Command::Kick { player } => {
                    players.remove_entry(&player);
                    let msg = Command::State {
                        players: players.keys().cloned().collect(),
                        state: self.state.clone(),
                    };
                    self.host.send_request(&msg);
                }
                Command::Stop() => {
                    self.state = GameState::Stopping;
                }
                Command::To { to, data } => {
                    for destination in to.iter() {
                        if let Some(player) = players.get_mut(destination) {
                            player.send_request(&data);
                        }
                    }
                }

                Command::Error { reason: _ } => {
                    self.host.send_request(&Command::Error {
                        reason: "Unhandled message".to_string(),
                    });
                }
                Command::PrepareReply { key: _ } => {
                    self.host.send_request(&Command::Error {
                        reason: "Unhandled message".to_string(),
                    });
                }
                Command::State {
                    players: _,
                    state: _,
                } => {
                    self.host.send_request(&Command::Error {
                        reason: "Unhandled message".to_string(),
                    });
                }
                Command::From { from: _, data: _ } => {
                    self.host.send_request(&Command::Error {
                        reason: "Unhandled message".to_string(),
                    });
                }
            }
        }

        if curr_state != self.state {
            let msg = Command::State {
                players: players.keys().cloned().collect(),
                state: self.state.clone(),
            };
            self.host.send_request(&msg);
            println!("{:?}", self.state);
        }
        true
    }
}
