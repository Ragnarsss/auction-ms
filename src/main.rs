mod grpc_server;
mod config;
mod models;
mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::init();
    grpc_server::start_grpc_server().await?;
    Ok(())
}