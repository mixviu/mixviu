//! Worker microbenchmark, excluding VFL decoding and I420 conversion.
use std::{hint::black_box, path::Path, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("usage: scaler TAKE.vfl")?;
    let mut reader = media_vfl::reader::Reader::open(Path::new(&path))?;
    let (w,h) = (reader.width() as usize,reader.height() as usize);
    let pixels = reader.seek(300)?.pixels.to_vec();
    let source = media_frame::Bgra::new(w,h,pixels)?;
    for filter in ["nearest","bicubic"] {
        let mut times = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            let image = if filter == "nearest" { source.scale(3840,2160)? } else { source.resample(3840,2160)? };
            black_box(image.pixels());
            times.push(start.elapsed().as_secs_f64()*1000.0);
        }
        times.sort_by(f64::total_cmp);
        println!("{}",serde_json::json!({"filter":filter,"runs":7,"median_ms":times[3],"min_ms":times[0],"max_ms":times[6]}));
    }
    Ok(())
}
