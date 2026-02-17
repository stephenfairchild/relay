use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

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

#[derive(Clone)]
struct CacheEntry {
    data: Bytes,
    cached_at: SystemTime,
}

type Cache = Arc<RwLock<HashMap<String, CacheEntry>>>;

// Metrics using lock-free atomic operations for minimal overhead
lazy_static! {
    static ref CACHE_HITS: IntCounter =
        register_int_counter!("relay_cache_hits_total", "Total number of cache hits").unwrap();
    static ref CACHE_MISSES: IntCounter =
        register_int_counter!("relay_cache_misses_total", "Total number of cache misses").unwrap();
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

#[derive(Debug, Deserialize, Clone)]
struct CacheConfig {
    #[serde(default = "default_ttl")]
    default_ttl: String,
    #[serde(default = "default_stale_while_revalidate")]
    stale_while_revalidate: String,
    #[serde(default = "default_stale_if_error")]
    stale_if_error: String,
    #[serde(default)]
    rules: HashMap<String, CacheRule>,
}

#[derive(Debug, Deserialize, Clone)]
struct CacheRule {
    #[serde(default)]
    ttl: Option<String>,
    #[serde(default)]
    stale: Option<String>,
}

fn default_ttl() -> String {
    "5m".to_string()
}

fn default_stale_while_revalidate() -> String {
    "1h".to_string()
}

fn default_stale_if_error() -> String {
    "24h".to_string()
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: default_ttl(),
            stale_while_revalidate: default_stale_while_revalidate(),
            stale_if_error: default_stale_if_error(),
            rules: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config("config.toml")?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let upstream_url = Arc::new(config.upstream.url.clone());
    let cache: Cache = Arc::new(RwLock::new(HashMap::new()));
    let prometheus_enabled = Arc::new(config.prometheus.enabled);
    let cache_config = Arc::new(config.cache.clone());

    println!("Server listening on {}", addr);
    println!("Upstream URL: {}", upstream_url);
    println!(
        "Prometheus metrics: {}",
        if *prometheus_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!("Cache default TTL: {}", cache_config.default_ttl);
    println!(
        "Cache stale-while-revalidate: {}",
        cache_config.stale_while_revalidate
    );
    println!("Cache stale-if-error: {}", cache_config.stale_if_error);
    if !cache_config.rules.is_empty() {
        println!("Cache rules: {} configured", cache_config.rules.len());
    }

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

fn parse_duration(s: &str) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration string".into());
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let number: u64 = if s.len() > 1 && s.chars().nth(s.len() - 2).unwrap_or('0').is_alphabetic() {
        // Handle two-letter units like "ms"
        let (num_str2, unit2) = s.split_at(s.len() - 2);
        if unit2 == "ms" {
            let n: u64 = num_str2.parse()?;
            return Ok(Duration::from_millis(n));
        }
        num_str.parse()?
    } else {
        num_str.parse()?
    };

    match unit {
        "s" => Ok(Duration::from_secs(number)),
        "m" => Ok(Duration::from_secs(number * 60)),
        "h" => Ok(Duration::from_secs(number * 3600)),
        "d" => Ok(Duration::from_secs(number * 86400)),
        _ => Err(format!("Unknown duration unit: {}", unit).into()),
    }
}

fn match_cache_rule<'a>(
    path: &str,
    rules: &'a HashMap<String, CacheRule>,
) -> Option<&'a CacheRule> {
    for (pattern, rule) in rules {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if path.starts_with(prefix) {
                return Some(rule);
            }
        } else if pattern == path {
            return Some(rule);
        }
    }
    None
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
    let path = incoming_uri.path();

    // Determine cache policy for this path
    let (ttl, stale_while_revalidate) =
        if let Some(rule) = match_cache_rule(path, &cache_config.rules) {
            let ttl = rule.ttl.as_ref().unwrap_or(&cache_config.default_ttl);
            let stale = rule
                .stale
                .as_ref()
                .unwrap_or(&cache_config.stale_while_revalidate);
            (
                parse_duration(ttl).unwrap_or(Duration::from_secs(300)),
                parse_duration(stale).unwrap_or(Duration::from_secs(3600)),
            )
        } else {
            (
                parse_duration(&cache_config.default_ttl).unwrap_or(Duration::from_secs(300)),
                parse_duration(&cache_config.stale_while_revalidate)
                    .unwrap_or(Duration::from_secs(3600)),
            )
        };

    let stale_if_error =
        parse_duration(&cache_config.stale_if_error).unwrap_or(Duration::from_secs(86400));

    // Check if the response is in cache
    let cached_entry = {
        let cache_read = cache.read().await;
        cache_read.get(&cache_key).cloned()
    };

    if let Some(ref entry) = cached_entry {
        let age = SystemTime::now()
            .duration_since(entry.cached_at)
            .unwrap_or(Duration::from_secs(0));

        // Check if cache is fresh
        if age <= ttl {
            if *prometheus_enabled {
                CACHE_HITS.inc();
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            println!(
                "Cache HIT (fresh): {} (age: {:?}, ttl: {:?})",
                cache_key, age, ttl
            );
            return Ok(Response::builder()
                .header("X-Cache", "HIT")
                .header("Age", age.as_secs().to_string())
                .body(Full::new(entry.data.clone()))?);
        }

        // Check if cache is stale but revalidatable
        if age <= ttl + stale_while_revalidate {
            if *prometheus_enabled {
                CACHE_HITS.inc();
            }
            println!(
                "Cache HIT (stale-while-revalidate): {} (age: {:?}, ttl: {:?})",
                cache_key, age, ttl
            );

            // Serve stale content immediately
            let response = Response::builder()
                .header("X-Cache", "STALE")
                .header("Age", age.as_secs().to_string())
                .body(Full::new(entry.data.clone()))?;

            // Trigger async revalidation (fire and forget)
            let upstream_url_clone = Arc::clone(&upstream_url);
            let cache_clone = Arc::clone(&cache);
            let cache_key_clone = cache_key.clone();
            let incoming_uri_clone = incoming_uri.clone();
            tokio::spawn(async move {
                if let Err(e) = revalidate_cache(
                    incoming_uri_clone,
                    upstream_url_clone,
                    cache_clone,
                    cache_key_clone,
                )
                .await
                {
                    eprintln!("Background revalidation failed: {:?}", e);
                }
            });

            if *prometheus_enabled {
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            return Ok(response);
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

    let address = format!("{}:{}", host, port);

    // Open a TCP connection to the remote host
    let stream = match TcpStream::connect(address).await {
        Ok(s) => s,
        Err(e) => {
            // On error, check if we can serve stale content
            if let Some(entry) = cached_entry {
                let age = SystemTime::now()
                    .duration_since(entry.cached_at)
                    .unwrap_or(Duration::from_secs(0));

                if age <= ttl + stale_if_error {
                    println!("Serving stale-if-error for: {} (age: {:?})", cache_key, age);
                    return Ok(Response::builder()
                        .header("X-Cache", "STALE-ERROR")
                        .header("Age", age.as_secs().to_string())
                        .body(Full::new(entry.data))?);
                }
            }

            if *prometheus_enabled {
                UPSTREAM_ERRORS.inc();
                REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
            }
            return Err(Box::new(e));
        }
    };

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
            // On error, check if we can serve stale content
            if let Some(entry) = cached_entry {
                let age = SystemTime::now()
                    .duration_since(entry.cached_at)
                    .unwrap_or(Duration::from_secs(0));

                if age <= ttl + stale_if_error {
                    println!("Serving stale-if-error for: {} (age: {:?})", cache_key, age);
                    return Ok(Response::builder()
                        .header("X-Cache", "STALE-ERROR")
                        .header("Age", age.as_secs().to_string())
                        .body(Full::new(entry.data))?);
                }
            }

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
        cache_write.insert(
            cache_key,
            CacheEntry {
                data: body_bytes.clone(),
                cached_at: SystemTime::now(),
            },
        );
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

async fn revalidate_cache(
    incoming_uri: hyper::Uri,
    upstream_url: Arc<String>,
    cache: Cache,
    cache_key: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_url = upstream_url.parse::<hyper::Uri>()?;
    let host = base_url.host().expect("uri has no host").to_string();
    let port = base_url.port_u16().unwrap_or(80);

    let path_and_query = incoming_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    let upstream_uri = format!(
        "{}://{}{}",
        base_url.scheme_str().unwrap_or("http"),
        base_url.authority().expect("uri has no authority"),
        path_and_query
    )
    .parse::<hyper::Uri>()?;

    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        let _ = conn.await;
    });

    let upstream_req = Request::builder()
        .uri(upstream_uri)
        .header(hyper::header::HOST, host)
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(upstream_req).await?;
    let body_bytes = res.collect().await?.to_bytes();

    let mut cache_write = cache.write().await;
    cache_write.insert(
        cache_key.clone(),
        CacheEntry {
            data: body_bytes,
            cached_at: SystemTime::now(),
        },
    );
    println!("Background revalidation complete for: {}", cache_key);

    Ok(())
}
