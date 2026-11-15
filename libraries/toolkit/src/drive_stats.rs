// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-written bounded frame samples. Reading never requests a repaint.
use serde_json::{Value, json};
use std::time::Instant;

// Shared storage contract with vendor/egui-wgpu/src/baseline.rs.
type Samples = (u64, Vec<f64>, Instant, [u32; 2], f32, String, String);

pub(crate) fn read(ctx: &egui::Context, viewport: egui::ViewportId) -> Value {
    let key = egui::Id::new(("ui.stats.v1", viewport));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn passive_reads_preserve_counts_and_never_wake() {
        let ctx = egui::Context::default();
        let wakes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = wakes.clone();
        ctx.set_request_repaint_callback(move |_| { observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst); });
        let viewport = egui::ViewportId::ROOT;
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(("ui.stats.v1", viewport)),
            (3_u64, vec![1.0_f64, 2.0, 9.0], Instant::now(), [640_u32, 480], 1.0_f32, "Vulkan".to_owned(), "fixture".to_owned())));
        let before = wakes.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(read(&ctx, viewport)["frames_drawn"], 3);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let stats = read(&ctx, viewport);
        assert_eq!(stats["frames_drawn"], 3);
        assert_eq!(stats["cpu_frame_ms"]["p95"], 9.0);
        assert!(stats["last_frame_ms"].as_f64().unwrap() >= 20.0);
        assert_eq!(wakes.load(std::sync::atomic::Ordering::SeqCst), before);
    }
}
