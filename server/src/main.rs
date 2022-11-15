use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::tungstenite::http::{StatusCode};
use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::{
    handshake::client::{Request, Response},
    Message, Result,
};
use tokio_tungstenite::{accept_async, tungstenite::Error};

use parking_lot::RwLock;

use once_cell::sync::Lazy;
use std::collections::HashMap;

// pub mod actor;
pub mod comm;
pub mod connection;
// pub mod game;

use comm::Command;

// use crate::actor::Actor;
// use crate::game::Game;
/*
static GAME_LIST: Lazy<RwLock<HashMap<String, RwLock<Game>>>> =
    Lazy::new(|| RwLock::new(HashMap::<String, RwLock<Game>>::default()));
*/

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

#[derive(Debug)]
enum ClientConfig {
    Connect(String),
    Create(String),
}

// Host -> Command -> Server -> Raw -> Client
//   ^_______Command____| ^______Raw_____|

async fn handle_connection(_peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut config: Arc<ClientConfig> = Arc::new(ClientConfig::Connect("".to_owned()));
    let ws_stream = accept_hdr_async(stream, |req: &Request, mut response: Response| {
        let res: Vec<&str> = req.uri().path().split("/").collect();

        if res.len() != 3 || res[2].len() != 4 {
            println!("{:?}", res);
            panic!("Rejected, less than 2 params or room id != 4 . How do we do this cleanly?");
        }
        if res[1] == "CONNECT" {
            config = Arc::new(ClientConfig::Connect(res[1].to_owned()));
        } else if res[1] == "CREATE" {
            config = Arc::new(ClientConfig::Create(res[1].to_owned()));
        } else {
            println!("{:?}", res);
            panic!("Rejected, type unknown. How do we do this cleanly?");
        }

        Ok(response)
    })
    .await
    .expect("Failed to accept");

    println!("{:?}", config);

    let (send_to_host, rx_to_be_sent_to_host) = unbounded_channel::<Command>();

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            ws_sender.send(msg).await?;
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                ws_sender.send(Message::Text("tick".to_owned())).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "127.0.0.1:8081";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }

    /*
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
                let map = GAME_LIST.read();
                let rw_game = map.get(&key);
                let game = rw_game.unwrap();
                let added: bool = game.write().add(websocket);
                if added {
                    println!("Added.");
                } else {
                    println!("Rejected.");
                }
            } else {
                {
                    println!("Creating the game.");
                    let mut map = GAME_LIST.write();
                    map.insert(key, RwLock::new(Game::new(Actor::new(0, websocket))));
                    println!("Created.");
                }
                spawn(move || {
                    println!("Running the game.");
                    let new_key: String = "key".to_string();
                    loop {
                        let ongoing = {
                            let map = GAME_LIST.read();
                            let rw_game = map.get(&new_key).unwrap();
                            let mut game = rw_game.write();
                            game.update()
                        };
                        if !ongoing {
                            let new_key: String = "key".to_string();
                            let mut map = GAME_LIST.write();
                            map.remove(&new_key).unwrap();
                            println!("Game finished.");
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                });
            }
        }

    */
}
