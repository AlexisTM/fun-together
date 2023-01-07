pub mod comm;
pub mod entry;
pub mod game;

#[cfg(feature = "tls")]
pub mod tls;

use crate::entry::service;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string())
        .parse()
        .unwrap();
    service(addr).await.unwrap();
}
