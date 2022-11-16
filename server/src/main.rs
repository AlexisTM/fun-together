use log::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::client::{Request, Response},
        Error, Result,
    },
};

use parking_lot::RwLock;

use once_cell::sync::Lazy;
use std::collections::HashMap;

pub mod comm;
pub mod game;
use game::{client_handler, game_handler};

use comm::{HostComm, Player};

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
}
