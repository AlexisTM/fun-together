
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::WebSocketStream;

use std::env;

use tokio_tungstenite::tungstenite::handshake::derive_accept_key;

use parking_lot::RwLock;

use once_cell::sync::Lazy;

pub mod comm;
pub mod game;
use game::{client_handler, game_handler, GameList};

use comm::{HostComm, Player};


use hyper::{
    header::{
        HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
        UPGRADE,
    },
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
    Body, Method, Request, Response, Server, StatusCode, Version,
};

static LAST_CLIENT_ID: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));

static GAME_LIST: Lazy<GameList> = Lazy::new(|| {
    Arc::new(RwLock::new(
        HashMap::<String, Arc<UnboundedSender<HostComm>>>::default(),
    ))
});

#[derive(Debug)]
enum ClientConfig {
    Connect(String),
    Create,
}

// We are handling a websocket connection and sprouting the game & players.
async fn handle_connection(
    _peer: SocketAddr,
    ws_stream: WebSocketStream<Upgraded>,
    client_id: u32,
    config: ClientConfig,
) {
    match config {
        ClientConfig::Connect(id) => {
            // If ID exists: Join to the game
            let to_game: Arc<UnboundedSender<HostComm>> =
                { GAME_LIST.read().get(&id).unwrap().clone() };
            tokio::spawn(client_handler(to_game, Player::new(client_id, ws_stream)));
        }
        ClientConfig::Create => {
            tokio::spawn(game_handler(ws_stream, GAME_LIST.clone()));
        }
    }
}

// Either reply in HTTP or upgrade to websocket
async fn handle_request(
    mut req: Request<Body>,
    addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    let new_client_id: u32 = {
        let mut id = LAST_CLIENT_ID.write();
        *id += 1;
        *id
    };

    let config: ClientConfig; // = Arc::new(ClientConfig::Connect("".to_owned()));
    let res: Vec<&str> = req.uri().path().split('/').collect();

    if res.len() != 2 {
        println!("{:?}", res);
        return Ok(Response::new(Body::from(
            "Either connect to a room or create one by connecting with a Websocket here.

GET to /ROOM will fetch information about the room
Connect to /ROOM will try to connect to the room
Connect to /CREATE will create a room",
        )));
    }
    if res[1].len() == 4 {
        config = ClientConfig::Connect(res[1].to_owned());
    } else if res[1] == "CREATE" {
        config = ClientConfig::Create;
    } else {
        println!("{:?}", res);
        return Ok(Response::new(Body::from(
            "Daaaaamn;;; You shouldn't be here",
        )));
    }

    let upgrade = HeaderValue::from_static("Upgrade");
    let websocket = HeaderValue::from_static("websocket");
    let headers = req.headers();
    let key = headers.get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));
    if req.method() != Method::GET
        || req.version() < Version::HTTP_11
        || !headers
            .get(CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|h| {
                h.split(|c| c == ' ' || c == ',')
                    .any(|p| p.eq_ignore_ascii_case(upgrade.to_str().unwrap()))
            })
            .unwrap_or(false)
        || !headers
            .get(UPGRADE)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false)
        || !headers
            .get(SEC_WEBSOCKET_VERSION)
            .map(|h| h == "13")
            .unwrap_or(false)
        || key.is_none()
    {
        // Return game name
        return Ok(Response::new(Body::from("Hello World!")));
    }
    let ver = req.version();

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                handle_connection(
                    addr,
                    WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await,
                    new_client_id,
                    config,
                )
                .await;
            }
            Err(e) => println!("upgrade error: {}", e),
        }
    });

    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    *res.version_mut() = ver;
    res.headers_mut().append(CONNECTION, upgrade);
    res.headers_mut().append(UPGRADE, websocket);
    res.headers_mut()
        .append(SEC_WEBSOCKET_ACCEPT, derived.unwrap().parse().unwrap());
    res.headers_mut()
        .append("ILoveRust", "Yes, I do.".parse().unwrap());
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    env_logger::init();

    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string())
        .parse()
        .unwrap();

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let service = service_fn(move |req| handle_request(req, remote_addr));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    server.await?;

    Ok::<_, hyper::Error>(())
}
