use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_epoch() -> f64 {
    let current_time = SystemTime::now();
    let current_time = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    current_time.as_secs() as f64 + current_time.subsec_nanos() as f64 / 1_000_000_000_f64
}
