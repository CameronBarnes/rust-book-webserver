use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    ops::Deref,
    sync::Arc,
    thread::{self},
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use codes::ResponseCode;
use request::{Method, Request};
use route::Routes;
use threadpool::ThreadPool;
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
    #[arg(short, long, default_value_t = 0)]
    threads: u8,
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
    routes.add_static("/", "static/hello.html", None)?;
    routes.set_404(route::Route::Static(
        "static/404.html".into(),
        Some(ResponseCode::Not_Found),
    ));
    routes.add_dynamic("/sleep", vec![Method::GET, Method::POST], |request| {
        let duration = request.body().map_or("5", |v| v).parse().unwrap_or(5);
        info!("Sleeping for {duration} seconds");
        thread::sleep(Duration::from_secs(duration));
        Ok(("Sleeping".into(), ResponseCode::Ok))
    })?;
    routes.set_static_dir("static/");
    routes.add_plain("/plain", "Test Plain", None)?;

    if args.threads == 1 {
        for stream in socket.incoming() {
            handle_connection(stream?, &routes);
        }
    } else {
        let routes = Arc::from(routes);
        let pool = if args.threads == 0 {
            ThreadPool::default()
        } else {
            ThreadPool::new(args.threads as usize)
        };
        for stream in socket.incoming() {
            let routes = routes.clone();
            pool.execute(move || handle_connection(stream.unwrap(), routes));
        }
    }

    Ok(())
}

fn handle_connection<R: Deref<Target = Routes>>(mut stream: TcpStream, routes: R) {
    let buf_reader = BufReader::new(&mut stream);
    let (content, code) = Request::parse(buf_reader).map_or_else(
        |_| ("Failed to parse".into(), ResponseCode::Bad_Request),
        |request| {
            debug!("Received Request:\n{}", &request.as_string());
            routes.apply(&request).unwrap()
        },
    );

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{content}",
        response::StatusLine::new(code),
        content.len(),
    );

    stream.write_all(response.as_bytes()).unwrap();
}
