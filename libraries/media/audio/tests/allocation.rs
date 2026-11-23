//! Fail a selected whole-programme allocation, without exhausting worker RAM.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
thread_local! { static FAIL_SIZE: Cell<usize> = const { Cell::new(0) }; }
struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if FAIL_SIZE
            .try_with(|size| {
                if size.get() != 0 && size.get() == layout.size() {
                    size.set(0);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false)
        {
            return std::ptr::null_mut();
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
#[test]
fn programme_and_limiter_allocation_failures_are_errors() {
    use media_audio::{Audio, WavFormat, timeline::Timeline};
    let mut audio = Audio::new(48000, 2, vec![0.1; 8192]).unwrap();
    FAIL_SIZE.with(|s| s.set(65536));
    assert!(audio.try_clone().is_err());
    FAIL_SIZE.with(|s| s.set(65536));
    assert!(
        audio
            .normalise_to_target(-14.0, -1.0, Default::default())
            .is_err()
    );
    assert!(audio.samples().iter().all(|s| *s == 0.1));
    let dir = std::env::temp_dir().join(format!("audio-allocation-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    Audio::new(48000, 1, vec![0.1; 4096])
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("source.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    let t:Timeline=serde_json::from_value(serde_json::json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":4096.0/48000.0,"sources":[{"path":"source.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
    FAIL_SIZE.with(|s| s.set(65536));
    assert!(t.mix(&dir).is_err());
    assert_eq!(
        FAIL_SIZE.with(Cell::get),
        0,
        "selected allocation was exercised"
    );
    std::fs::remove_dir_all(dir).unwrap();
}
