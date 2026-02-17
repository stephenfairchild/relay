use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

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
use tokio::sync::RwLock;

use lazy_static::lazy_static;
use prometheus::{Encoder, Histogram, IntCounter, IntGauge, TextEncoder, register_histogram, register_int_counter, register_int_gauge};

type Cache = Arc<RwLock<HashMap<String, Bytes>>>;

// Metrics using lock-free atomic operations for minimal overhead
lazy_static! {
    static ref CACHE_HITS: IntCounter = register_int_counter!(
        "relay_cache_hits_total",
        "Total number of cache hits"
    ).unwrap();

    static ref CACHE_MISSES: IntCounter = register_int_counter!(
        "relay_cache_misses_total",
        "Total number of cache misses"
    ).unwrap();

    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "relay_request_duration_seconds",
        "Request duration in seconds",
        vec![0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5]
    ).unwrap();

    static ref UPSTREAM_ERRORS: IntCounter = register_int_counter!(
        "relay_upstream_errors_total",
        "Total number of upstream request errors"
    ).unwrap();

    static ref CACHE_SIZE: IntGauge = register_int_gauge!(
        "relay_cache_entries",
        "Current number of entries in cache"
    ).unwrap();
}

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    upstream: UpstreamConfig,
    #[serde(default)]
    prometheus: PrometheusConfig,
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

#[derive(Debug, Deserialize)]
struct PrometheusConfig {
    #[serde(default)]
    enabled: bool,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config("config.toml")?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let upstream_url = Arc::new(config.upstream.url.clone());
    let cache: Cache = Arc::new(RwLock::new(HashMap::new()));
    let prometheus_enabled = Arc::new(config.prometheus.enabled);

    println!("Server listening on {}", addr);
    println!("Upstream URL: {}", upstream_url);
    println!("Prometheus metrics: {}", if *prometheus_enabled { "enabled" } else { "disabled" });

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let upstream_url = Arc::clone(&upstream_url);
        let cache = Arc::clone(&cache);
        let prometheus_enabled = Arc::clone(&prometheus_enabled);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        handle_request(
                            req,
                            Arc::clone(&upstream_url),
                            Arc::clone(&cache),
                            Arc::clone(&prometheus_enabled),
                        )
                    }),
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

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    upstream_url: Arc<String>,
    cache: Cache,
    prometheus_enabled: Arc<bool>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    // Fast path check for metrics endpoint
    if req.uri().path() == "/metrics" {
        if *prometheus_enabled {
            return metrics_handler().await;
        } else {
            return Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))?);
        }
    }

    call_upstream(req, upstream_url, cache, prometheus_enabled).await
}

async fn metrics_handler() -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;

    Ok(Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Full::new(Bytes::from(buffer)))?)
}

fn generate_cache_key(uri: &hyper::Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string()
}

async fn call_upstream(
    req: Request<hyper::body::Incoming>,
    upstream_url: Arc<String>,
    cache: Cache,
    prometheus_enabled: Arc<bool>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let incoming_uri = req.uri().clone();
    let cache_key = generate_cache_key(&incoming_uri);

    // Check if the response is in cache
    {
        let cache_read = cache.read().await;
        if let Some(cached_bytes) = cache_read.get(&cache_key) {
            if *prometheus_enabled {
                CACHE_HITS.inc();
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            println!("Cache HIT: {}", cache_key);
            return Ok(Response::builder()
                .header("X-Cache", "HIT")
                .body(Full::new(cached_bytes.clone()))?);
        }
    }

    if *prometheus_enabled {
        CACHE_MISSES.inc();
    }
    println!("Cache MISS: {}", cache_key);

    let base_url = upstream_url.parse::<hyper::Uri>()?;

    // Get the host and the port from the upstream URL
    let host = base_url.host().expect("uri has no host").to_string();
    let port = base_url.port_u16().unwrap_or(80);

    // Get the path and query from the incoming request
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
    let res = match sender.send_request(upstream_req).await {
        Ok(r) => r,
        Err(e) => {
            if *prometheus_enabled {
                UPSTREAM_ERRORS.inc();
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            return Err(Box::new(e));
        }
    };

    // Read the response body
    let body_bytes = res.collect().await?.to_bytes();

    // Store the response in cache
    {
        let mut cache_write = cache.write().await;
        cache_write.insert(cache_key, body_bytes.clone());
        if *prometheus_enabled {
            CACHE_SIZE.set(cache_write.len() as i64);
        }
    }

    if *prometheus_enabled {
        REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
    }

    // Return the response with cache miss header
    Ok(Response::builder()
        .header("X-Cache", "MISS")
        .body(Full::new(body_bytes))?)
}
