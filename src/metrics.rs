use lazy_static::lazy_static;
use prometheus::{
    Histogram, IntCounter, IntGauge, register_histogram, register_int_counter, register_int_gauge,
};

lazy_static! {
    pub static ref CACHE_HITS: IntCounter =
        register_int_counter!("relay_cache_hits_total", "Total number of cache hits").unwrap();
    pub static ref CACHE_MISSES: IntCounter =
        register_int_counter!("relay_cache_misses_total", "Total number of cache misses").unwrap();
    pub static ref CACHE_STALE_SERVED: IntCounter = register_int_counter!(
        "relay_cache_stale_served_total",
        "Total number of stale cache responses served"
    )
    .unwrap();
    pub static ref REQUEST_DURATION: Histogram = register_histogram!(
        "relay_request_duration_seconds",
        "Request duration in seconds",
        vec![
            0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5
        ]
    )
    .unwrap();
    pub static ref UPSTREAM_ERRORS: IntCounter = register_int_counter!(
        "relay_upstream_errors_total",
        "Total number of upstream request errors"
    )
    .unwrap();
    pub static ref CACHE_SIZE: IntGauge =
        register_int_gauge!("relay_cache_entries", "Current number of entries in cache").unwrap();
}
