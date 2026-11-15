// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-written bounded frame samples. Reading never requests a repaint.
use serde_json::{Value, json};
use std::time::Instant;

// Shared storage contract with vendor/egui-wgpu/src/baseline.rs.
type Samples = (u64, Vec<f64>, Instant, [u32; 2], f32, String, String);

pub(crate) fn read(ctx: &egui::Context) -> Value {
    let key = egui::Id::new(("ui.stats.v1", ctx.viewport_id()));
    let Some((frames, mut times, last, size, scale, backend, adapter)) =
        ctx.data(|d| d.get_temp::<Samples>(key))
    else { return json!({"available": false}); };
    times.sort_by(f64::total_cmp);
    let percentile = |p: f64| times[((times.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    json!({"available":true,"frames_drawn":frames,"sample_count":times.len(),
        "cpu_frame_ms":{"median":percentile(0.5),"p95":percentile(0.95),"max":times.last()},
        "last_frame_ms":last.elapsed().as_secs_f64()*1000.0,"surface_pixels":size,
        "scale":scale,"backend":backend,"adapter":adapter})
}
