use futures::{select, FutureExt};
use futures_util::{SinkExt, StreamExt};

use std::borrow::Cow;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Error, Message, Result};
use tokio_tungstenite::WebSocketStream;

use std::collections::HashMap;

use crate::comm::{Command, HostComm, Player, PlayerSink};

fn read_command(msg: Option<Result<Message, Error>>) -> Option<Command> {
    if let Some(Ok(msg)) = msg {
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
    None
}

fn to_message(command: Command) -> Message {
    let val = serde_json::to_string(&command).unwrap();
    Message::Text(val)
}

fn to_state(connections: &HashMap<u32, PlayerSink>) -> Command {
    Command::State {
        players: connections.keys().cloned().collect(),
    }
}

// Broadcast all the incoming game state to the clients.
// One game handler per game
pub async fn game_handler(
    mut rx: UnboundedReceiver<HostComm>,
    mut host: WebSocketStream<TcpStream>,
) {
    let mut connections: HashMap<u32, PlayerSink> = HashMap::new();
    let mut max_players_: u32 = 0;

    let mut accept_players = false;

    loop {
        select! {
            event = rx.recv().fuse() => {
                if let Some(event) = event {
                    // host.send(event).await;
                    match event {
                        HostComm::Join(conn) => {
                            if accept_players {
                                let _success = connections.insert(conn.id, conn);
                            } else {
                                // How to close it?
                            }
                            host.send(to_message(to_state(&connections))).await.unwrap();
                        }
                        HostComm::Leave(conn) => {
                            connections.remove(&conn);
                            host.send(to_message(to_state(&connections))).await.unwrap();
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
                        Command::Prepare{max_players} => {
                            max_players_ = max_players;
                            accept_players = true;
                            host.send(to_message(to_state(&connections))).await.unwrap();
                        },
                        Command::Start() => {
                            if connections.len() <= max_players_.try_into().unwrap() {
                                accept_players = false;
                            }
                            host.send(to_message(to_state(&connections))).await.unwrap();
                        },
                        Command::Kick{player} => {
                            connections.remove(&player);
                            host.send(to_message(to_state(&connections))).await.unwrap();
                        },
                        Command::Stop() => {
                            let close_msg = Some(CloseFrame{code: CloseCode::Away, reason: Cow::Borrowed("The game is done.")});
                            if let Ok(_) = host.close(close_msg).await {
                                // Cool
                            }
                            let keys: Vec<_> = connections.keys().cloned().collect();
                            for connection in keys.iter() {
                                let val = connections.remove(connection);
                                if let Some(mut val) = val {
                                    if let Ok(_)  = val.sink.close().await {
                                        // Cool
                                    }
                                }
                            }
                            break;
                        },
                        Command::To { to, data } => {
                            for player in to.iter() {
                                if let Some(dest) = connections.get_mut(player) {
                                    dest.sink.send(Message::Text(data.clone())).await.unwrap();
                                }
                            }
                        },
                        _ => {},
                    }
                }
            },
        }
    }
}

// One client handler per client;
pub async fn client_handler(game_sender: Arc<UnboundedSender<HostComm>>, player: Player) {
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
