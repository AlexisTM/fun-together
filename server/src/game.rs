use std::{sync::RwLock};

use crate::actor::Actor;

pub struct Game {
    name: String,
    host: Actor,
    players: Vec<Actor>,
}

impl Game {
    pub fn new(name: String, host: Actor) -> Self {
        Self {
            name,
            host,
            players: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_host(&mut self) -> &mut Actor {
        &mut self.host
    }

    pub fn add(&mut self, player: Actor) {
        self.players.push(player);
    }
}
