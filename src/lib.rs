pub mod comm;
pub mod entry;
pub mod game;

#[cfg(feature = "tls")]
pub mod tls;

use crate::entry::service;

struct PoolService {}

#[shuttle_service::async_trait]
impl shuttle_service::Service for PoolService {
    async fn bind(
        mut self: Box<Self>,
        addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_service::error::Error> {
        service(addr).await.unwrap();
        Ok(())
    }
}

#[shuttle_service::main]
async fn init() -> Result<PoolService, shuttle_service::Error> {
    Ok(PoolService {})
}
