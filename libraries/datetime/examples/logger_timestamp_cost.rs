// SPDX-License-Identifier: MIT OR Apache-2.0
//! CBC microbenchmark of noded's empty-event.ts timestamp/line formatting path.
//! Excludes Bus and disk I/O; keep a second thread alive like the logger runtime.
use datetime::Local;
use std::{hint::black_box, time::Instant};
fn main() {
    let (release, wait) = std::sync::mpsc::channel();
    let other = std::thread::spawn(move || wait.recv().unwrap());
    for sample in 0..5 {
        let started = Instant::now();
        for _ in 0..100_000 {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            black_box(format!(
                "[{now}] source → destination  command  wire={}B\n",
                black_box(128)
            ));
        }
        println!(
            "LOGGER_TIMESTAMP_SAMPLE={sample} iterations=100000 elapsed_ns={}",
            started.elapsed().as_nanos()
        );
    }
    release.send(()).unwrap();
    other.join().unwrap();
}
