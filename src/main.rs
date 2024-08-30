use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;
use clap::Parser;
use request::Request;
use route::Routes;
use tracing::{debug, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod codes;
mod request;
mod response;
mod route;

pub static SUPPORTED_HTTP_VERSION: &str = "HTTP/1.1";

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

    let mut routes = Routes::default();
    routes.add_static("/", "static/hello.html")?;

    for stream in socket.incoming() {
        debug!("Incomming connection!");
        handle_connection(stream?, &routes)?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, routes: &Routes) -> Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let request = Request::parse(buf_reader)?; // FIXME: We should handle this error here

    //debug!("Received Request:\n{}", format_request(&request));

    let (content, code) = routes.apply(&request)?;

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{content}",
        response::StatusLine::new(code),
        content.len(),
    );

    stream.write_all(response.as_bytes())?;

    Ok(())
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
