use std::{net::TcpListener, thread::spawn, time::Duration};

use parking_lot::RwLock;

use once_cell::sync::Lazy;
use std::collections::HashMap;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

pub mod actor;
pub mod comm;
pub mod game;

use crate::actor::Actor;

use crate::game::Game;

static GAME_LIST: Lazy<RwLock<HashMap<String, RwLock<Game>>>> =
    Lazy::new(|| RwLock::new(HashMap::<String, RwLock<Game>>::default()));

fn main() {
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:8081").unwrap();
    for stream in server.incoming() {
        let callback = |req: &Request, mut response: Response| {
            println!("Received a new ws handshake");
            println!("The request's path is: {}", req.uri().path());
            println!("The request's headers are:");
            for (ref header, _value) in req.headers() {
                println!("* {}", header);
            }

            // Let's add an additional header to our response to the client.
            let headers = response.headers_mut();
            headers.append("MyCustomHeader", ":)".parse().unwrap());
            headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

            Ok(response)
        };

        let key: String = "key".to_string();
        let _name: String = "ID_OF_GAME".to_string();
        let _host_name: String = "host".to_string();
        let _user_name: String = "user".to_string();

        let s = stream.unwrap();
        s.set_nonblocking(true).unwrap();
        let websocket_res = accept_hdr(s, callback);
        if websocket_res.is_err() {
            return;
        }
        let websocket = websocket_res.unwrap();

        let available: bool = {
            let map = GAME_LIST.read();
            map.contains_key(&(key.clone()))
        };
        if available {
            println!("Adding a new user.");
            // STUCK;
            let map = GAME_LIST.read();
            let rw_game = map.get(&key);
            let game = rw_game.unwrap();
            game.write().add(websocket);
            println!("Added.");
        } else {
            {
                println!("Creating the game.");
                let mut map = GAME_LIST.write();
                map.insert(
                    key,
                    RwLock::new(Game::new(
                        Actor::new(0, websocket),
                    )),
                );
                println!("Created.");
            }
            spawn(move || {
                println!("Starting a new game.");
                loop {
                    {
                        let new_key: String = "key".to_string();
                        let map = GAME_LIST.read();
                        let rw_game = map.get(&new_key).unwrap();
                        let mut game = rw_game.write();
                        game.update();
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });
        }
    }
}
