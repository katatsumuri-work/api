use std::error::Error;

use katatsumuri_api::{AppConfig, build_app};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env()?;
    let app = build_app(&config);
    let listener = TcpListener::bind(&config.bind_addr).await?;
    let addr = listener.local_addr()?;

    tracing::info!(%addr, "katatsumuri-api リスニング開始");

    axum::serve(listener, app).await?;
    Ok(())
}
