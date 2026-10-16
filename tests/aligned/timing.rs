//! Optional benchmark instrumentation. File I/O happens after the timer stops.
use std::{io::Write, sync::Mutex, time::Instant};
static FILE_LOCK: Mutex<()> = Mutex::new(());
fn record(id: &str, elapsed: u128) {
    if let Some(path) = std::env::var_os("NEAKASA_TEST_TIMINGS") {
        let _lock = FILE_LOCK.lock().unwrap();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(file, "{}", serde_json::json!({"id":id,"ns":elapsed})).unwrap();
    }
}
pub fn sync(id: &str, body: impl FnOnce()) {
    let start = Instant::now();
    body();
    record(id, start.elapsed().as_nanos());
}
pub async fn asynchronous(id: &str, body: impl std::future::Future<Output = ()>) {
    let start = Instant::now();
    body.await;
    record(id, start.elapsed().as_nanos());
}
