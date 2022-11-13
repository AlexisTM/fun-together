use std::sync::RwLockWriteGuard;

use crate::comm::{GameRequest, GameState, GameResponseWithSource};
use crate::actor::Actor;

pub fn game_handler(
    _state: &mut GameState,
    host: &mut RwLockWriteGuard<Actor>,
    _players: &mut RwLockWriteGuard<Vec<Actor>>,
    _messages: &Vec<GameResponseWithSource>,
) {
    let request = GameRequest::default();
    host.send_request(&request);
}
