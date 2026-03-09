
pub mod api;
pub mod domain;
pub mod infra;
pub mod app;

pub use crate::app::state::AppState;
pub use crate::infra::config::{
    Settings, DatabaseSettings, RedisSettings, LogSettings, ServerSettings, S3Settings,
};

pub mod ids {
    use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static LAST_MS: AtomicU64 = AtomicU64::new(0);
    static SEQ: AtomicU16 = AtomicU16::new(0);
    const EPOCH_MS: u64 = 1577836800000;

    fn now_ms() -> u64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis() as u64,
            Err(_) => 0,
        }
    }

    pub fn snowflake() -> u64 {
        let worker_id = (std::process::id() as u64) & 0x03FF;
        loop {
            let current = now_ms();
            let last = LAST_MS.load(Ordering::Relaxed);
            if current == last {
                let seq = SEQ.fetch_add(1, Ordering::Relaxed) & 0x0FFF;
                if seq != 0 {
                    return ((current - EPOCH_MS) << 22) | (worker_id << 12) | seq as u64;
                }
            } else if current > last {
                LAST_MS.store(current, Ordering::Relaxed);
                SEQ.store(0, Ordering::Relaxed);
                let seq = SEQ.fetch_add(1, Ordering::Relaxed) & 0x0FFF;
                return ((current - EPOCH_MS) << 22) | (worker_id << 12) | seq as u64;
            } else {
                let seq = SEQ.fetch_add(1, Ordering::Relaxed) & 0x0FFF;
                if seq != 0 {
                    return ((last - EPOCH_MS) << 22) | (worker_id << 12) | seq as u64;
                }
            }
            std::thread::yield_now();
        }
    }

    pub fn generate_prefixed_snowflake(prefix: &str) -> String {
        format!("{}{}", prefix, snowflake())
    }
}
