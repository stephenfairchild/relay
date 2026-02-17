use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;

use crate::cache::CachedResponse;
use crate::config::CacheConfig;
use crate::metrics::{
    CACHE_HITS, CACHE_MISSES, CACHE_SIZE, CACHE_STALE_SERVED, REQUEST_DURATION, UPSTREAM_ERRORS,
};
use crate::storage::Cache;

pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    upstream_url: Arc<String>,
    cache: Cache,
    prometheus_enabled: Arc<bool>,
    cache_config: Arc<CacheConfig>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
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

pub async fn metrics_handler(
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;

    Ok(Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Full::new(Bytes::from(buffer)))?)
}

pub fn generate_cache_key(uri: &hyper::Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string()
}

pub async fn call_upstream(
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

    // Check if this path has a cache rule
    let rule = cache_config.find_rule(path);

    // If bypass is enabled for this path, skip caching entirely
    if let Some(rule) = rule {
        if rule.bypass == Some(true) {
            println!("Cache BYPASS: {cache_key}");
            return forward_to_upstream(
                req,
                upstream_url,
                incoming_uri,
                prometheus_enabled,
                start,
            )
            .await;
        }
    }

    // Determine TTL to use (rule-specific or default)
    let ttl = rule
        .and_then(|r| r.ttl)
        .unwrap_or(cache_config.default_ttl);

    if let Some(cached_response) = cache.get(&cache_key).await {
        if !cached_response.is_stale(ttl) {
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

    let address = format!("{host}:{port}");

    let stream = TcpStream::connect(address).await?;

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {err:?}");
        }
    });

    let upstream_req = Request::builder()
        .uri(upstream_uri)
        .header(hyper::header::HOST, host)
        .body(Empty::<Bytes>::new())?;

    let res = match sender.send_request(upstream_req).await {
        Ok(r) => r,
        Err(e) => {
            if *prometheus_enabled {
                UPSTREAM_ERRORS.inc();
            }

            // Determine stale duration to use (rule-specific or default)
            let stale_if_error = rule
                .and_then(|r| r.stale)
                .unwrap_or(cache_config.stale_if_error);

            if let Some(cached_response) = cache.get(&cache_key).await {
                if cached_response.is_servable_if_error(ttl, stale_if_error) {
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

    let body_bytes = res.collect().await?.to_bytes();

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

    Ok(Response::builder()
        .header("X-Cache", "MISS")
        .body(Full::new(body_bytes))?)
}

async fn forward_to_upstream(
    _req: Request<hyper::body::Incoming>,
    upstream_url: Arc<String>,
    incoming_uri: hyper::Uri,
    prometheus_enabled: Arc<bool>,
    start: Instant,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
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

    let address = format!("{host}:{port}");
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {err:?}");
        }
    });

    let upstream_req = Request::builder()
        .uri(upstream_uri)
        .header(hyper::header::HOST, host)
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(upstream_req).await?;
    let body_bytes = res.collect().await?.to_bytes();

    if *prometheus_enabled {
        REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
    }

    Ok(Response::builder()
        .header("X-Cache", "BYPASS")
        .body(Full::new(body_bytes))?)
}
