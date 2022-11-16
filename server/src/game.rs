use futures::{select, FutureExt};

use futures_util::{SinkExt, StreamExt};

use std::sync::Arc;
use tokio::net::{TcpStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::{
    Message, Result,
};
use tokio_tungstenite::{WebSocketStream};

use std::collections::HashMap;

use crate::comm::{Command, HostComm, Player, PlayerSink};

pub fn read_command(msg: Option<Result<Message, Error>>) -> Option<Command> {
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
            _event = rx.recv().fuse() => {
                if let Some(event) = rx.recv().await {
                    // host.send(event).await;
                    match event {
                        HostComm::Join(conn) => {
                            if accept_players {
                                let _success = connections.insert(conn.id, conn);
                                let val = serde_json::to_string(&Command::State {
                                    players: connections.keys().cloned().collect(),
                                });
                                host.send(Message::Text(val.unwrap())).await.unwrap();
                            } else {
                                // How to close it?
                            }
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
