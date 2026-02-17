use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::BodyExt;
use http_body_util::Empty;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    upstream: UpstreamConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct UpstreamConfig {
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config("config.toml")?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let upstream_url = Arc::new(config.upstream.url.clone());

    println!("Server listening on {}", addr);
    println!("Upstream URL: {}", upstream_url);

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let upstream_url = Arc::clone(&upstream_url);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| call_upstream(req, Arc::clone(&upstream_url))),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

async fn call_upstream(
    req: Request<hyper::body::Incoming>,
    upstream_url: Arc<String>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = upstream_url.parse::<hyper::Uri>()?;

    // Get the host and the port from the upstream URL
    let host = base_url.host().expect("uri has no host").to_string();
    let port = base_url.port_u16().unwrap_or(80);

    // Get the path and query from the incoming request
    let incoming_uri = req.uri();
    let path_and_query = incoming_uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    // Construct the full upstream URL with path and query
    let upstream_uri = format!(
        "{}://{}{}",
        base_url.scheme_str().unwrap_or("http"),
        base_url.authority().expect("uri has no authority"),
        path_and_query
    ).parse::<hyper::Uri>()?;

    let address = format!("{}:{}", host, port);

    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address).await?;

    // Use an adapter to access something implementing `tokio::io` traits as if they implement
    // `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    // Create the request to send to the upstream server
    let upstream_req = Request::builder()
        .uri(upstream_uri)
        .header(hyper::header::HOST, host)
        .body(Empty::<Bytes>::new())?;

    // Send the request and await the response
    let res = sender.send_request(upstream_req).await?;

    // Read the response body
    let body_bytes = res.collect().await?.to_bytes();

    // Return the response
    Ok(Response::new(Full::new(body_bytes)))
}
