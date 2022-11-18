use log::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedSender;

use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::client::{Request, Response},
        Error, Result,
    },
};

use parking_lot::RwLock;

use once_cell::sync::Lazy;

pub mod comm;
pub mod game;
use game::{client_handler, game_handler, GameList};

use comm::{HostComm, Player};

static GAME_LIST: Lazy<GameList> = Lazy::new(|| {
    Arc::new(RwLock::new(
        HashMap::<String, Arc<UnboundedSender<HostComm>>>::default(),
    ))
});

async fn accept_connection(peer: SocketAddr, stream: TcpStream, client_id: u32) {
    if let Err(e) = handle_connection(peer, stream, client_id).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

#[derive(Debug)]
enum ClientConfig {
    Connect(String),
    Create,
}

// Host -> Command -> Server -> Raw -> Client
//   ^_______Command____| ^______Raw_____|

async fn handle_connection(_peer: SocketAddr, stream: TcpStream, client_id: u32) -> Result<()> {
    let mut config: Arc<ClientConfig> = Arc::new(ClientConfig::Connect("".to_owned()));
    let ws_stream = accept_hdr_async(stream, |req: &Request, response: Response| {
        let res: Vec<&str> = req.uri().path().split('/').collect();

        if res.len() != 2 {
            // res[2].len() != 4 {
            println!("{:?}", res);
            panic!("Rejected, less than 2 params or room id != 4 . How do we do this cleanly?");
        }
        if res[1].len() == 4 {
            config = Arc::new(ClientConfig::Connect(res[1].to_owned()));
        } else if res[1] == "CREATE" {
            config = Arc::new(ClientConfig::Create);
        } else {
            println!("{:?}", res);
            panic!("Rejected, type unknown. How do we do this cleanly?");
        }
        Ok(response)
    })
    .await
    .expect("Failed to accept");

    match &*config {
        ClientConfig::Connect(id) => {
            // If ID exists: Join to the game
            let to_game: Arc<UnboundedSender<HostComm>> =
                { GAME_LIST.read().get(id).unwrap().clone() };
            tokio::spawn(client_handler(to_game, Player::new(client_id, ws_stream)));
        }
        ClientConfig::Create => {
            tokio::spawn(game_handler(ws_stream, GAME_LIST.clone()));
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

    let mut client_id: u32 = 0;
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        client_id += 1;
        tokio::spawn(accept_connection(peer, stream, client_id));
    }
}
