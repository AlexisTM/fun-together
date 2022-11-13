use std::sync::RwLockWriteGuard;

use crate::comm::{GameRequest, GameResponseWithSource, GameAction};
use crate::actor::Actor;

pub fn game_handler(
    host: &mut RwLockWriteGuard<Actor>,
    _players: &mut RwLockWriteGuard<Vec<Actor>>,
    _messages: &[GameResponseWithSource],
) -> bool {
    let mut request = GameRequest::default();
    request.action = GameAction::RequestText;
    request.title = "Give me some fun".to_string();
    request.description = "Example: I love fried chicken!".to_string();
    request.id = 1000;
    host.send_request(&request);
    return true;
}
