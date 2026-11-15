// SPDX-License-Identifier: MIT OR Apache-2.0
//! Passive frame observations shared with toolkit's ui.stats reader.
use std::time::Instant;
type Samples = (u64, Vec<f64>, Instant, [u32; 2], f32, String, String);

/// Record completed host frame work, without scheduling any work.
pub fn record(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    seconds: f64,
    size: [u32; 2],
    scale: f32,
    adapter: &wgpu::AdapterInfo,
) {
    let key = egui::Id::unique(("ui.stats.v1", viewport));
    ctx.data_mut(|d| {
        let s = d.get_temp_mut_or_insert_with::<Samples>(key, || {
            (
                0,
                Vec::with_capacity(256),
                Instant::now(),
                size,
                scale,
                format!("{:?}", adapter.backend),
                adapter.name.clone(),
            )
        });
        s.0 += 1;
        if s.1.len() == 256 {
            s.1.remove(0);
        }
        s.1.push(seconds * 1000.0);
        s.2 = Instant::now();
        s.3 = size;
        s.4 = scale;
    });
}
