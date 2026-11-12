// SPDX-License-Identifier: MIT OR Apache-2.0
// CBC-only fresh-process startup comparison; each prefix has its own env.mx.
use std::{path::Path, process::Command, time::Instant};

fn sample(prefix: &Path) -> u128 {
    let start = Instant::now();
    let result = Command::new(prefix.join("bin/mix"))
        .args(["-c", "print(1)"])
        .env_clear()
        .env("HOME", prefix)
        .env("MIX_PREFIX", prefix)
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C")
        .current_dir(prefix)
        .output()
        .expect("start mix");
    let elapsed = start.elapsed().as_nanos();
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, b"1\n");
    elapsed
}

fn main() {
    let prefixes: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(prefixes.len(), 2);
    let mut samples = [Vec::new(), Vec::new()];
    for round in 0..101 {
        for index in [round % 2, 1 - round % 2] {
            samples[index].push(sample(Path::new(&prefixes[index])));
        }
    }
    for values in &mut samples {
        values.sort_unstable();
    }
    let before = samples[0][50];
    let after = samples[1][50];
    println!(
        "STARTUP before_median_ns={before} after_median_ns={after} samples=101 alternating=true"
    );
    assert!(
        after <= before + 1_000_000,
        "median startup grew by more than 1 ms"
    );
}
