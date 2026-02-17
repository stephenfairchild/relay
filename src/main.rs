use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
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
use prometheus::{
    Encoder, Histogram, IntCounter, IntGauge, TextEncoder, register_histogram,
    register_int_counter, register_int_gauge,
};

#[derive(Clone, Debug)]
struct CachedResponse {
    body: Bytes,
    cached_at: Instant,
}

impl CachedResponse {
    fn is_stale(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }

    fn is_servable_if_error(&self, ttl: Duration, stale_if_error: Duration) -> bool {
        self.cached_at.elapsed() < ttl + stale_if_error
    }
}

#[async_trait]
trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Option<CachedResponse>;
    async fn set(&self, key: String, value: CachedResponse);
    async fn size(&self) -> usize;
}

struct MemoryStorage {
    cache: RwLock<HashMap<String, CachedResponse>>,
}

impl MemoryStorage {
    fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Option<CachedResponse> {
        self.cache.read().await.get(key).cloned()
    }

    async fn set(&self, key: String, value: CachedResponse) {
        self.cache.write().await.insert(key, value);
    }

    async fn size(&self) -> usize {
        self.cache.read().await.len()
    }
}

struct RedisStorage {
    client: redis::aio::ConnectionManager,
}

impl RedisStorage {
    async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let connection_manager = redis::aio::ConnectionManager::new(client).await?;
        Ok(Self {
            client: connection_manager,
        })
    }
}

#[async_trait]
impl Storage for RedisStorage {
    async fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut conn = self.client.clone();

        let result: Result<(Vec<u8>, u64), redis::RedisError> = redis::pipe()
            .get(format!("{key}:body"))
            .get(format!("{key}:cached_at"))
            .query_async(&mut conn)
            .await;

        match result {
            Ok((body, cached_at_nanos)) => {
                let elapsed = Duration::from_nanos(cached_at_nanos);
                let cached_at = Instant::now() - elapsed;
                Some(CachedResponse {
                    body: Bytes::from(body),
                    cached_at,
                })
            }
            Err(_) => None,
        }
    }

    async fn set(&self, key: String, value: CachedResponse) {
        let mut conn = self.client.clone();
        let elapsed = value.cached_at.elapsed().as_nanos() as u64;

        let _: Result<(), redis::RedisError> = redis::pipe()
            .set(format!("{key}:body"), value.body.to_vec())
            .set(format!("{key}:cached_at"), elapsed)
            .query_async(&mut conn)
            .await;
    }

    async fn size(&self) -> usize {
        0
    }
}

type Cache = Arc<dyn Storage>;

// Metrics using lock-free atomic operations for minimal overhead
lazy_static! {
    static ref CACHE_HITS: IntCounter =
        register_int_counter!("relay_cache_hits_total", "Total number of cache hits").unwrap();
    static ref CACHE_MISSES: IntCounter =
        register_int_counter!("relay_cache_misses_total", "Total number of cache misses").unwrap();
    static ref CACHE_STALE_SERVED: IntCounter = register_int_counter!(
        "relay_cache_stale_served_total",
        "Total number of stale cache responses served"
    )
    .unwrap();
    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "relay_request_duration_seconds",
        "Request duration in seconds",
        vec![
            0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5
        ]
    )
    .unwrap();
    static ref UPSTREAM_ERRORS: IntCounter = register_int_counter!(
        "relay_upstream_errors_total",
        "Total number of upstream request errors"
    )
    .unwrap();
    static ref CACHE_SIZE: IntGauge =
        register_int_gauge!("relay_cache_entries", "Current number of entries in cache").unwrap();
}

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    upstream: UpstreamConfig,
    #[serde(default)]
    prometheus: PrometheusConfig,
    #[serde(default)]
    cache: CacheConfig,
    #[serde(default)]
    storage: StorageConfig,
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

#[derive(Debug, Deserialize, Default)]
struct PrometheusConfig {
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct CacheConfig {
    #[serde(default = "default_ttl", deserialize_with = "deserialize_duration")]
    default_ttl: Duration,
    #[serde(
        default = "default_stale_if_error",
        deserialize_with = "deserialize_duration"
    )]
    stale_if_error: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: default_ttl(),
            stale_if_error: default_stale_if_error(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StorageConfig {
    #[serde(default = "default_backend")]
    backend: String,
    redis: Option<RedisConfig>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            redis: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RedisConfig {
    url: String,
}

fn default_backend() -> String {
    "memory".to_string()
}

fn default_ttl() -> Duration {
    Duration::from_secs(300) // 5 minutes
}

fn default_stale_if_error() -> Duration {
    Duration::from_secs(86400) // 24 hours
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_duration(&s).map_err(serde::de::Error::custom)
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Duration string is empty".to_string());
    }

    let (value_str, unit) = s.split_at(s.len() - 1);
    let last_char = s.chars().last().unwrap();

    let (num_str, unit_str) = if last_char.is_alphabetic() {
        (value_str, unit)
    } else {
        (s, "s")
    };

    let value: u64 = num_str
        .parse()
        .map_err(|_| format!("Invalid number: {num_str}"))?;

    let multiplier = match unit_str {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => return Err(format!("Invalid time unit: {unit_str}")),
    };

    Ok(Duration::from_secs(value * multiplier))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config("config.toml")?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let upstream_url = Arc::new(config.upstream.url.clone());

    let cache: Cache = match config.storage.backend.as_str() {
        "redis" => {
            let redis_config = config
                .storage
                .redis
                .as_ref()
                .ok_or("Redis backend selected but no redis configuration provided")?;
            println!("Initializing Redis storage backend: {}", redis_config.url);
            Arc::new(RedisStorage::new(&redis_config.url).await?)
        }
        "memory" => {
            println!("Initializing in-memory storage backend");
            Arc::new(MemoryStorage::new())
        }
        backend => {
            return Err(format!("Unknown storage backend: {backend}").into());
        }
    };

    let prometheus_enabled = Arc::new(config.prometheus.enabled);
    let cache_config = Arc::new(config.cache);

    println!("Server listening on {addr}");
    println!("Upstream URL: {upstream_url}");
    println!(
        "Prometheus metrics: {}",
        if *prometheus_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    let ttl = cache_config.default_ttl;
    let stale_if_error = cache_config.stale_if_error;
    println!("Cache config: TTL={ttl:?}, stale-if-error={stale_if_error:?}");

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let upstream_url = Arc::clone(&upstream_url);
        let cache = Arc::clone(&cache);
        let prometheus_enabled = Arc::clone(&prometheus_enabled);
        let cache_config = Arc::clone(&cache_config);

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
                            Arc::clone(&cache_config),
                        )
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {err:?}");
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
    cache_config: Arc<CacheConfig>,
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

    call_upstream(req, upstream_url, cache, prometheus_enabled, cache_config).await
}

async fn metrics_handler() -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>>
{
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
    cache_config: Arc<CacheConfig>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let incoming_uri = req.uri().clone();
    let cache_key = generate_cache_key(&incoming_uri);

    // Check if the response is in cache
    if let Some(cached_response) = cache.get(&cache_key).await {
        if !cached_response.is_stale(cache_config.default_ttl) {
            if *prometheus_enabled {
                CACHE_HITS.inc();
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            println!("Cache HIT: {cache_key}");
            return Ok(Response::builder()
                .header("X-Cache", "HIT")
                .body(Full::new(cached_response.body.clone()))?);
        }
    }

    if *prometheus_enabled {
        CACHE_MISSES.inc();
    }
    println!("Cache MISS: {cache_key}");

    let base_url = upstream_url.parse::<hyper::Uri>()?;

    // Get the host and the port from the upstream URL
    let host = base_url.host().expect("uri has no host").to_string();
    let port = base_url.port_u16().unwrap_or(80);

    // Get the path and query from the incoming request
    let path_and_query = incoming_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    // Construct the full upstream URL with path and query
    let upstream_uri = format!(
        "{}://{}{}",
        base_url.scheme_str().unwrap_or("http"),
        base_url.authority().expect("uri has no authority"),
        path_and_query
    )
    .parse::<hyper::Uri>()?;

    let address = format!("{host}:{port}");

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
            println!("Connection failed: {err:?}");
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
            }

            // Check if we have stale cache that can be served
            if let Some(cached_response) = cache.get(&cache_key).await {
                if cached_response
                    .is_servable_if_error(cache_config.default_ttl, cache_config.stale_if_error)
                {
                    if *prometheus_enabled {
                        CACHE_STALE_SERVED.inc();
                        REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
                    }
                    println!(
                        "Cache STALE (serving due to upstream error): {cache_key} - error: {e}"
                    );
                    return Ok(Response::builder()
                        .header("X-Cache", "STALE")
                        .header("X-Cache-Reason", "upstream-error")
                        .body(Full::new(cached_response.body.clone()))?);
                }
            }

            if *prometheus_enabled {
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            return Err(Box::new(e));
        }
    };

    // Read the response body
    let body_bytes = res.collect().await?.to_bytes();

    // Store the response in cache
    cache
        .set(
            cache_key.clone(),
            CachedResponse {
                body: body_bytes.clone(),
                cached_at: Instant::now(),
            },
        )
        .await;

    if *prometheus_enabled {
        CACHE_SIZE.set(cache.size().await as i64);
    }

    if *prometheus_enabled {
        REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
    }

    // Return the response with cache miss header
    Ok(Response::builder()
        .header("X-Cache", "MISS")
        .body(Full::new(body_bytes))?)
}
