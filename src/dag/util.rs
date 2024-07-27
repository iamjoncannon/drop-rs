use std::time::Duration;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

pub fn current_time() -> u128 {
    let start = SystemTime::now();

    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis()
}
