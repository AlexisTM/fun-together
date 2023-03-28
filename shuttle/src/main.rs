use fun_together::entry;

use entry::service;
struct PoolService {}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for PoolService {
    async fn bind(
        mut self: Self,
        addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_runtime::Error> {
        service(addr).await.unwrap();
        Ok(())
    }
}

#[shuttle_runtime::main]
async fn init() -> Result<PoolService, shuttle_runtime::Error> {
    Ok(PoolService {})
}
