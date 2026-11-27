// SPDX-License-Identifier: MIT OR Apache-2.0
use super::ProviderWarning;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Log {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn missing_provider_warns_at_startup_and_after_loss_with_a_rate_limit() {
    let log = Log::default();
    let writer = log.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let now = Instant::now();
        let mut warning = ProviderWarning::default();
        warning.observe(true, "4", now);
        assert!(log.0.lock().unwrap().is_empty());
        warning.observe(false, "4", now);
        warning.observe(false, "4", now + Duration::from_secs(5));
        warning.observe(true, "4", now + Duration::from_secs(10));
        warning.observe(false, "4", now + Duration::from_secs(15));
        assert_eq!(String::from_utf8_lossy(&log.0.lock().unwrap()).matches("WARN").count(), 1);
        warning.observe(false, "4", now + Duration::from_secs(60));
        let bytes = log.0.lock().unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert_eq!(text.matches("WARN").count(), 2);
        assert_eq!(text.matches("start files@4.service").count(), 2);
    });
}
