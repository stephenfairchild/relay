use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::LoggingConfig;

#[derive(Debug, Clone, Copy)]
pub enum CacheStatus {
    Hit,
    Miss,
    Bypass,
    Stale,
}

impl CacheStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheStatus::Hit => "HIT",
            CacheStatus::Miss => "MISS",
            CacheStatus::Bypass => "BYPASS",
            CacheStatus::Stale => "STALE",
        }
    }
}

pub struct AccessLogEntry {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: f64,
    pub cache_status: CacheStatus,
    pub remote_addr: SocketAddr,
    pub bytes_sent: usize,
}

pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !config.enabled {
        return Ok(());
    }

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        "combined" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().with_target(false).with_level(false))
                .init();
        }
        _ => {
            return Err(format!("Invalid log format: {}", config.format).into());
        }
    }

    Ok(())
}

pub fn log_access(entry: AccessLogEntry) {
    info!(
        method = %entry.method,
        path = %entry.path,
        status = entry.status,
        duration_ms = entry.duration_ms,
        cache_status = entry.cache_status.as_str(),
        remote_addr = %entry.remote_addr,
        bytes_sent = entry.bytes_sent,
        "access"
    );
}
