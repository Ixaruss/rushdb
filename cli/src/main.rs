mod client;
mod cmd;
mod libc;
mod shell;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cmd::Cmd::init().await;
    Ok(())
}
