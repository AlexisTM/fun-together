use futures::{select, FutureExt};


use futures_util::{SinkExt, StreamExt};
use log::*;
use std::sync::Arc;
use std::{net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use tokio_tungstenite::tungstenite::{
    handshake::client::{Request, Response},
    Message, Result,
};
use tokio_tungstenite::{tungstenite::Error};
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};

use parking_lot::RwLock;

use once_cell::sync::Lazy;
use std::collections::HashMap;

// pub mod actor;
pub mod comm;
// pub mod game;

use comm::{Command};
use comm::{HostComm, Player, PlayerSink};

// use crate::actor::Actor;
// use crate::game::Game;
/*
static GAME_LIST: Lazy<RwLock<HashMap<String, RwLock<Game>>>> =
    Lazy::new(|| RwLock::new(HashMap::<String, RwLock<Game>>::default()));
*/

static GAME_LIST: Lazy<RwLock<HashMap<String, Arc<UnboundedSender<HostComm>>>>> =
    Lazy::new(|| RwLock::new(HashMap::<String, Arc<UnboundedSender<HostComm>>>::default()));

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
    let ws_stream = accept_hdr_async(stream, |req: &Request, response: Response| {
        let res: Vec<&str> = req.uri().path().split('/').collect();

        if res.len() != 3 || res[2].len() != 4 {
            println!("{:?}", res);
            panic!("Rejected, less than 2 params or room id != 4 . How do we do this cleanly?");
        }
        if res[1] == "CONNECT" {
            config = Arc::new(ClientConfig::Connect(res[2].to_owned()));
        } else if res[1] == "CREATE" {
            config = Arc::new(ClientConfig::Create(res[2].to_owned()));
        } else {
            println!("{:?}", res);
            panic!("Rejected, type unknown. How do we do this cleanly?");
        }
        Ok(response)
    })
    .await
    .expect("Failed to accept");

    println!("{:?}", config);

    match &*config {
        ClientConfig::Connect(id) => {
            // If ID exists: Join to the game
            let to_game: Arc<UnboundedSender<HostComm>> =
                { GAME_LIST.read().get(id).unwrap().clone() };
            tokio::spawn(client_handler(to_game, Player::new(1, ws_stream)));
        }
        ClientConfig::Create(id) => {
            let (to_game, game_cmd_receiver) = unbounded_channel::<HostComm>();
            {
                GAME_LIST.write().insert(id.to_string(), Arc::new(to_game));
            }
            tokio::spawn(game_handler(game_cmd_receiver, ws_stream));
        }
    }
    // Should there be something here?
    Ok(())
}

pub fn read_command(msg: Option<Result<Message, Error>>) -> Option<Command> {
    if let Some(msg) = msg {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(x) => {
                    let json: Result<Command, _> = serde_json::from_str(x.as_str());
                    match json {
                        Ok(valid_json) => return Some(valid_json),
                        Err(_) => return None,
                    }
                }
                Message::Close(_x) => {
                    return Some(Command::Stop());
                }
                _ => return None,
            }
        }
    }
    None
}

// Broadcast all the incoming game state to the clients.
// One game handler per game
async fn game_handler(mut rx: UnboundedReceiver<HostComm>, mut host: WebSocketStream<TcpStream>) {
    let mut connections: HashMap<u32, PlayerSink> = HashMap::new();
    let mut max_players_: u32 = 0;

    // Lobby
    // Playing
    // Stopping

    let mut accept_players = false;

    loop {
        select! {
            _event = rx.recv().fuse() => {
                if let Some(event) = rx.recv().await {
                    // host.send(event).await;
                    match event {
                        HostComm::Join(conn) => {
                            let _success = connections.insert(conn.id, conn);
                            let val = serde_json::to_string(&Command::State {
                                players: connections.keys().cloned().collect(),
                            });
                            host.send(Message::Text(val.unwrap())).await.unwrap();
                        }
                        HostComm::Leave(conn) => {
                            connections.remove(&conn);
                            let val = serde_json::to_string(&Command::State {
                                players: connections.keys().cloned().collect(),
                            });
                            host.send(Message::Text(val.unwrap())).await.unwrap();
                        }
                        HostComm::Command(cmd) => {
                            let val = serde_json::to_string(&cmd);
                            host.send(Message::Text(val.unwrap())).await.unwrap();
                        }
                    }
                }
            },
            event = host.next().fuse() => {
                let cmd = read_command(event);
                if let Some(cmd) = cmd {
                    match cmd {
                        Command::Prepare{max_players} => {
                            max_players_ = max_players;
                            accept_players = true;
                        },
                        Command::Start() => {
                            if connections.len() <= max_players_.try_into().unwrap() {
                                accept_players = false;
                            }
                        },
                        Command::Kick{player} => {
                            connections.remove(&player);
                        },
                        Command::Stop() => {},
                        Command::To { to: _, data: _ } => {}, // Send to user
                        _ => {},
                    }
                }
            },
            // complete => break,
            // default => unreachable!(),
        }
    }
}

// One client handler per client;
async fn client_handler(game_sender: Arc<UnboundedSender<HostComm>>, player: Player) {
    let (sink, mut stream) = player.ws.split();
    game_sender
        .send(HostComm::Join(PlayerSink::new(1, sink)))
        .unwrap();

    while let Some(msg) = stream.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(str_data) = msg {
                let _ = game_sender.send(HostComm::Command(Command::From {
                    from: player.id,
                    data: str_data,
                }));
            } else if msg.is_close() {
                break; // When we break, we disconnect.
            }
        } else {
            break; // When we break, we disconnect.
        }
    }
    game_sender.send(HostComm::Leave(player.id)).unwrap();
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
