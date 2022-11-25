use futures::{select, FutureExt};
use futures_util::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use log::info;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Error, Message, Result};
use tokio_tungstenite::WebSocketStream;

use parking_lot::RwLock;

use crate::comm::{Command, HostComm, Player, PlayerSink};

use ciborium;

pub struct GameConfig {
    pub to_game: Arc<UnboundedSender<HostComm>>,
    pub name: String,
}

pub type GameList = Arc<RwLock<HashMap<String, GameConfig>>>;

use rand::thread_rng;
use rand::Rng;

fn gen_room_code() -> String {
    let mut rng = thread_rng();
    let mut val: String = "".to_owned();
    for _ in 0..4 {
        let random_val: u8 = rng.gen_range(65..91);
        val.push(random_val as char);
    }
    val
}

fn read_command(msg: Option<Result<Message, Error>>) -> Option<Command> {
    if let Some(Ok(msg)) = msg {
        match msg {
            Message::Text(x) => {
                let val: Result<Command, _> = serde_json::from_str(x.as_str());
                match val {
                    Ok(valid_json) => return Some(valid_json),
                    Err(_) => return None,
                }
            }
            Message::Binary(x) => {
                let val: Result<Command, _> = ciborium::de::from_reader(x.as_slice());
                // let val: Result<Command, _> = serde_json::from_str(x.as_str());
                match val {
                    Ok(valid_json) => return Some(valid_json),
                    Err(_) => return None,
                }
            }
            Message::Close(_x) => {
                return Some(Command::Stop);
            }
            _ => return None,
        }
    }
    None
}

fn to_message(command: Command) -> Message {
    let mut buf: Vec<u8> = Vec::new();
    let res = ciborium::ser::into_writer(&command, &mut buf);
    if res.is_ok() {}
    Message::Binary(buf)
}

#[allow(dead_code)]
fn to_json(command: Command) -> Message {
    let val = serde_json::to_string(&command).unwrap();
    Message::Text(val)
}

fn to_state(
    name: &str,
    connections: &HashMap<u32, PlayerSink>,
    max_players: u32,
    accept_conns: bool,
) -> Command {
    let players: Vec<u32> = connections.keys().cloned().collect();
    let accept_connections = accept_conns && players.len() < max_players.try_into().unwrap();
    Command::State {
        name: name.to_string(),
        players,
        max_players,
        accept_conns: accept_connections,
    }
}

// Broadcast all the incoming game state to the clients.
// One game handler per game
pub async fn game_handler(mut host: WebSocketStream<Upgraded>, game_list: GameList) {
    let mut connections: HashMap<u32, PlayerSink> = HashMap::new();
    let mut max_players_: u32 = 0;

    let mut accept_players = false;

    let (tx_to_here, mut rx) = unbounded_channel::<HostComm>();
    let tx_to_here = Arc::new(tx_to_here);

    let mut game_name = "".to_owned();

    info!("A game started.");
    host.send(to_message(to_state(
        &game_name,
        &connections,
        max_players_,
        accept_players,
    )))
    .await
    .unwrap();

    host.send(to_message(Command::To {
        to: vec![1, 2, 3],
        data: vec![1, 2, 3],
    }))
    .await
    .unwrap();

    let mut id: Option<String> = None;

    loop {
        select! {
            event = rx.recv().fuse() => {
                if let Some(event) = event {
                    match event {
                        HostComm::Join(mut conn) => {
                            if accept_players && connections.len() < max_players_.try_into().unwrap() {
                                let _success = connections.insert(conn.id, conn);
                            } else if (conn.sink.close().await).is_ok() {}
                            host.send(to_message(to_state(&game_name,&connections, max_players_, accept_players))).await.unwrap();
                        }
                        HostComm::Leave(conn) => {
                            connections.remove(&conn);
                            host.send(to_message(to_state(&game_name,&connections, max_players_, accept_players))).await.unwrap();
                        }
                        HostComm::Command(cmd) => {
                            host.send(to_message(cmd)).await.unwrap();
                        }
                    }
                }
            },
            event = host.next().fuse() => {
                let cmd = read_command(event);
                if let Some(cmd) = cmd {
                    match cmd {
                        Command::Prepare{max_players, name} => {
                            if id.is_none() {
                                max_players_ = max_players;
                                accept_players = true;
                                game_name = name.clone();
                                host.send(to_message(to_state(&game_name,&connections, max_players_, accept_players))).await.unwrap();

                                for _ in 0..4 {
                                    let val = gen_room_code();
                                    if !game_list.read().contains_key(&val) {
                                        id = Some(val);
                                        break;
                                    }
                                }

                                if let Some(room) = id.clone() {
                                    info!("The new game id is {:?}.", id);
                                    host.send(to_message(Command::PrepareReply { key: room.clone() } )).await.unwrap();
                                    {
                                        game_list.write().insert(room, GameConfig{
                                            name: game_name.clone(),
                                            to_game: tx_to_here.clone(),
                                        });
                                    }
                                } else {
                                    host.send(to_message(Command::Error { reason: "Failed to find a unique room key".to_owned() })).await.unwrap();
                                }
                            } else if let Some(room) = id.clone() {
                                host.send(to_message(Command::PrepareReply { key: room } )).await.unwrap();
                            }
                        },
                        Command::Start => {
                            if connections.len() <= max_players_.try_into().unwrap() {
                                accept_players = false;
                            }
                            host.send(to_message(to_state(&game_name,&connections, max_players_, accept_players))).await.unwrap();
                        },
                        Command::Kick{player} => {
                            let conn = connections.remove(&player);
                            if let Some(mut conn) = conn {
                                conn.sink.close().await.unwrap();
                            }
                            host.send(to_message(to_state(&game_name,&connections, max_players_, accept_players))).await.unwrap();
                        },
                        Command::Stop => {
                            break;
                        },
                        Command::To { to, data } => {
                            let dest: Vec<u32>;
                            if to.is_empty() {
                                dest = connections.keys().cloned().collect();
                            } else {
                                dest = to;
                            }
                            for player in dest.iter() {
                                if let Some(dest) = connections.get_mut(player) {
                                    dest.sink.send(Message::Binary(data.clone())).await.unwrap();
                                }
                            }
                        },
                        Command::ToStr { to, data } => {
                            let dest: Vec<u32>;
                            if to.is_empty() {
                                dest = connections.keys().cloned().collect();
                            } else {
                                dest = to;
                            }
                            for player in dest.iter() {
                                if let Some(dest) = connections.get_mut(player) {
                                    dest.sink.send(Message::Text(data.clone())).await.unwrap();
                                }
                            }
                        },
                        _ => {},
                    }
                } else {
                    host.send(to_message(Command::Error{reason: "Invalid message.".to_owned()})).await.unwrap();
                }
            },
            complete => {break;}
        }
    }

    info!("Game {:?} finished.", id);
    let close_msg = Some(CloseFrame {
        code: CloseCode::Away,
        reason: Cow::Borrowed("The game is done."),
    });
    if let Ok(_) = host.close(close_msg).await {
        // Cool
    }
    let keys: Vec<_> = connections.keys().cloned().collect();
    for connection in keys.iter() {
        let val = connections.remove(connection);
        if let Some(mut val) = val {
            if let Ok(_) = val.sink.close().await {
                // Cool
            }
        }
    }
    {
        if let Some(room) = id.clone() {
            game_list.write().remove(&room);
        }
    }
}

// One client handler per client;
pub async fn client_handler(game_sender: Arc<UnboundedSender<HostComm>>, player: Player) {
    let (sink, mut stream) = player.ws.split();

    game_sender
        .send(HostComm::Join(PlayerSink::new(player.id, sink)))
        .unwrap();

    while let Some(msg) = stream.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(str_data) = msg {
                let _ = game_sender.send(HostComm::Command(Command::FromStr {
                    from: player.id,
                    data: str_data,
                }));
            } else if let Message::Binary(data) = msg {
                let _ = game_sender.send(HostComm::Command(Command::From {
                    from: player.id,
                    data,
                }));
            } else if msg.is_close() {
                break;
            }
        } else {
            break;
        }
    }
    // If this fails, the game is already finished.
    game_sender.send(HostComm::Leave(player.id)).unwrap();
}
