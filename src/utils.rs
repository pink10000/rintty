use std::time::Duration;

pub fn last_n_chars(s: &str, n: usize) -> &str {
    let len = s.len();
    if len <= n {
        s
    } else {
        &s[len - n..]
    }
}

pub fn calculate_tick_rate(max_framerate: Option<u64>) -> Duration {
    if let Some(max_framerate) = max_framerate {
        Duration::from_millis(1000 / max_framerate)
    } else {
        Duration::from_millis(1000 / 60)
    }
}