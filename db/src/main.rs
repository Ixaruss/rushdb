use crate::server::server_init;
pub mod conf;
pub mod engine;
pub mod persistence;
pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server_init().await?;

    Ok(())
}
