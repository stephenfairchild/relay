use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub prometheus: PrometheusConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct PrometheusConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_logging_enabled")]
    pub enabled: bool,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: default_logging_enabled(),
            format: default_log_format(),
        }
    }
}

fn default_logging_enabled() -> bool {
    true
}

fn default_log_format() -> String {
    "json".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheRule {
    #[serde(default, deserialize_with = "deserialize_optional_duration")]
    pub ttl: Option<Duration>,
    #[serde(default, deserialize_with = "deserialize_optional_duration")]
    pub stale: Option<Duration>,
    #[serde(default)]
    pub bypass: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_ttl", deserialize_with = "deserialize_duration")]
    pub default_ttl: Duration,
    #[serde(
        default = "default_stale_if_error",
        deserialize_with = "deserialize_duration"
    )]
    pub stale_if_error: Duration,
    #[serde(default)]
    pub rules: Option<HashMap<String, CacheRule>>,
    #[serde(skip)]
    pub compiled_rules: Option<Vec<(GlobSet, CacheRule)>>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: default_ttl(),
            stale_if_error: default_stale_if_error(),
            rules: None,
            compiled_rules: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    pub redis: Option<RedisConfig>,
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
pub struct RedisConfig {
    pub url: String,
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

fn deserialize_optional_duration<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => parse_duration(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
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

impl CacheConfig {
    pub fn compile_rules(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(rules) = &self.rules {
            let mut compiled = Vec::new();
            for (pattern, rule) in rules {
                let mut builder = GlobSetBuilder::new();
                builder.add(Glob::new(pattern)?);
                let globset = builder.build()?;
                compiled.push((globset, rule.clone()));
            }
            self.compiled_rules = Some(compiled);
        }
        Ok(())
    }

    pub fn find_rule(&self, path: &str) -> Option<&CacheRule> {
        if let Some(compiled) = &self.compiled_rules {
            for (globset, rule) in compiled {
                if globset.is_match(path) {
                    return Some(rule);
                }
            }
        }
        None
    }
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let config_str = std::fs::read_to_string(path)?;
    let mut config: Config = toml::from_str(&config_str)?;
    config.cache.compile_rules()?;
    Ok(config)
}
