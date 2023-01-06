use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{sync::mpsc::UnboundedSender};
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::WebSocketStream;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::comm::{HostComm, Player};
use crate::game::{client_handler, game_handler, GameConfig, GameList};

#[cfg(feature = "tls")]
use tls::get_tls_cfg;
#[cfg(feature = "tls")]
use hyper::server::conn::AddrIncoming;

use hyper::{
    header::{
        HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
        UPGRADE,
    },
    server::conn::{AddrStream},
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
    Body, Method, Request, Response, Server, StatusCode, Version,
};

static LAST_CLIENT_ID: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));

static GAME_LIST: Lazy<GameList> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::<String, GameConfig>::default())));

#[derive(Debug, PartialEq)]
enum ClientConfig {
    Connect(String),
    Create,
    Invalid,
}

// We are handling a websocket connection and sprouting the game & players.
async fn handle_connection(
    ws_stream: WebSocketStream<Upgraded>,
    client_id: u32,
    config: ClientConfig,
) {
    match config {
        ClientConfig::Connect(id) => {
            // If ID exists: Join to the game
            let to_game: Arc<UnboundedSender<HostComm>> =
                { GAME_LIST.read().get(&id).unwrap().to_game.clone() };
            tokio::spawn(client_handler(to_game, Player::new(client_id, ws_stream)));
        }
        ClientConfig::Create => {
            tokio::spawn(game_handler(ws_stream, GAME_LIST.clone()));
        }
        ClientConfig::Invalid => {
            panic!("We tried to start a connection for an invalid client.")
        }
    }
}

// Either reply in HTTP or upgrade to websocket
async fn handle_request(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let new_client_id: u32 = {
        let mut id = LAST_CLIENT_ID.write();
        *id += 1;
        *id
    };

    let config: ClientConfig; // = Arc::new(ClientConfig::Connect("".to_owned()));
    let res: Vec<&str> = req.uri().path().split('/').collect();

    if res.len() != 2 {
        config = ClientConfig::Invalid;
    } else if res[1].len() == 4 {
        config = ClientConfig::Connect(res[1].to_owned());
    } else if res[1] == "CREATE" {
        config = ClientConfig::Create;
    } else {
        config = ClientConfig::Invalid;
    }

    let upgrade = HeaderValue::from_static("Upgrade");
    let websocket = HeaderValue::from_static("websocket");
    let headers = req.headers();
    let key = headers.get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));
    if config == ClientConfig::Invalid
        || req.method() != Method::GET
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
        // Handle the request if we don't want to level up to Websocket mode.
        match &config {
            ClientConfig::Connect(str) => {
                if let Some(val) = GAME_LIST.read().get(str) {
                    return Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Body::from(val.name.clone()))
                        .unwrap());
                } else {
                    return Ok(Response::builder()
                        .status(404)
                        .header("Content-Type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Body::from("Game not found"))
                        .unwrap());
                }
            }
            ClientConfig::Invalid => {
                return Ok(Response::builder()
                    .status(400)
                    .header("Content-Type", "text/plain")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Body::from(
                        "Either connect to a room or create one by connecting with a Websocket here.

    GET to /ROOM will fetch information about the room
    Connect to /ROOM will try to connect to the room
    Connect to /CREATE will create a room",
                    ))
                    .unwrap());
            }
            _ => {
                return Ok(Response::new(Body::from("Provide a valid token.")));
            }
        }
    }
    let ver = req.version();

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                handle_connection(
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


pub async fn service(addr: SocketAddr) -> Result<(), hyper::Error> {
    #[cfg(feature = "tls")]
    let server = {
        let incoming = AddrIncoming::bind(&addr)?;
        let tls_cfg = get_tls_cfg();
        let make_svc = make_service_fn(move |_conn: &tls::TlsStream| {
            let service = service_fn(handle_request);
            async move { Ok::<_, Infallible>(service) }
        });
        Server::builder(tls::TlsAcceptor::new(tls_cfg, incoming)).serve(make_svc)
    };

    #[cfg(not(feature = "tls"))]
    let server = {
        let make_svc = make_service_fn(move |_: &AddrStream| {
            let service = service_fn(handle_request);
            async move { Ok::<_, Infallible>(service) }
        });
        Server::bind(&addr).serve(make_svc)
    };

    server.await?;

    Ok::<_, hyper::Error>(())
}
