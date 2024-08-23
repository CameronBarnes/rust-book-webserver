use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    address: String,
    #[arg(short, long, default_value_t = String::from("0"))]
    port: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    human_panic::setup_panic!();

    tracing_subscriber::registry().with(fmt::layer()).init();

    info!("Starting Webserver");
    let socket = TcpListener::bind(format!("{}:{}", args.address, args.port))?;
    let address = socket.local_addr()?;
    info!("Socket bound to address: {}", &address);

    for stream in socket.incoming() {
        handle_connection(stream?);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request = buf_reader
        .lines()
        .map_while(Result::ok)
        .take_while(|line| !line.is_empty())
        .collect_vec();

    info!("Received Request:\n{}", format_request(&request));
}

fn format_request(request: &[String]) -> String {
    let mut out = String::new();
    for str in request {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(str);
    }
    out
}
