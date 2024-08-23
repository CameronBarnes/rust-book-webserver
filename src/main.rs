use std::net::TcpListener;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    human_panic::setup_panic!();

    tracing_subscriber::registry().with(fmt::layer()).init();

    info!("Starting Webserver");
    let socket = TcpListener::bind("127.0.0.1:7878")?;

    for stream in socket.incoming() {
        let _ = stream?;
        info!("Connection received!");
    }

    Ok(())
}
