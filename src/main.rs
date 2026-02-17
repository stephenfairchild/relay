mod cache;
mod config;
mod handlers;
mod metrics;
mod storage;

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use config::load_config;
use handlers::handle_request;
use storage::{Cache, MemoryStorage, RedisStorage};

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

    if let Some(rules) = &cache_config.rules {
        println!("Cache rules configured:");
        for (pattern, rule) in rules {
            if let Some(true) = rule.bypass {
                println!("  {pattern} -> BYPASS");
            } else {
                println!("  {pattern} -> TTL={:?}, stale={:?}", rule.ttl, rule.stale);
            }
        }
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
                eprintln!("Error serving connection: {err:?}");
            }
        });
    }
}
