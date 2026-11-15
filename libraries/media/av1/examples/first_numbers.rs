//! Generated fixtures and IVF envelope for worker interop, not a product muxer.
use media_av1::{Encoder, Settings, Target};
use media_frame::Bgra;
use std::io::Write;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let w: usize = args[2].parse().unwrap();
    let h: usize = args[3].parse().unwrap();
    let speed: u8 = args[4].parse().unwrap();
    let scene = &args[5];
    let dir = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(dir).unwrap();
    let mut enc = Encoder::new(Settings {
        width: w,
        height: h,
        fps_num: 30,
        fps_den: 1,
        speed,
        target: Target::Quality(100),
        key_interval: 30,
        tile_cols: 2,
        tile_rows: 2,
        threads: 2,
    })
    .unwrap();
    println!("actual_tiles={:?}", enc.tiles());
    let mut source = std::fs::File::create(dir.join("source.yuv")).unwrap();
    let mut frames = Vec::new();
    for t in 0..8 {
        let mut pixels = vec![0; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let source_y = if scene == "scroll" { y + t * 13 } else { y };
                let row = source_y / 28;
                let mut c = if x < 200 {
                    [42, 38, 34, 255]
                } else if row % 2 == 0 {
                    [75, 65, 55, 255]
                } else {
                    [55, 48, 42, 255]
                };
                // Coloured glyph-like one-pixel strokes, including sharp edges.
                if x > 220 && x < 900 && source_y % 28 > 8 && source_y % 28 < 20 && (x + row * 3) % 11 < 2 {
                    c = [220, 180, 85, 255];
                }
                if scene == "static" && t >= 4 && x > 930 && x < 933 && (80..104).contains(&y) {
                    c = [240, 240, 240, 255];
                }
                if scene == "drag"
                    && x > 300 + t * 30
                    && x < 1000 + t * 30
                    && y > h / 3
                    && y < h / 3 + 350
                {
                    c = if y < h / 3 + 25 {
                        [150, 80, 45, 255]
                    } else {
                        [220, 220, 220, 255]
                    };
                }
                pixels[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(&c);
            }
        }
        let f = Bgra::new(w, h, pixels).unwrap().to_yuv420();
        for p in [&f.y, &f.u, &f.v] {
            source.write_all(p).unwrap();
        }
        frames.push(enc.prepare(&f).unwrap());
    }
    let start = std::time::Instant::now();
    let mut packets = Vec::new();
    for f in &frames {
        packets.extend(enc.submit(f).unwrap());
    }
    packets.extend(enc.finish().unwrap());
    let secs = start.elapsed().as_secs_f64();
    assert_eq!(packets.len(), 8);
    let mut ivf = std::fs::File::create(dir.join("encoded.ivf")).unwrap();
    ivf.write_all(b"DKIF").unwrap();
    ivf.write_all(&0u16.to_le_bytes()).unwrap();
    ivf.write_all(&32u16.to_le_bytes()).unwrap();
    ivf.write_all(b"AV01").unwrap();
    ivf.write_all(&(w as u16).to_le_bytes()).unwrap();
    ivf.write_all(&(h as u16).to_le_bytes()).unwrap();
    for n in [30u32, 1, 8, 0] {
        ivf.write_all(&n.to_le_bytes()).unwrap();
    }
    let mut bytes = 0;
    for p in packets {
        bytes += p.data.len();
        ivf.write_all(&(p.data.len() as u32).to_le_bytes()).unwrap();
        ivf.write_all(&p.pts.to_le_bytes()).unwrap();
        ivf.write_all(&p.data).unwrap();
    }
    println!(
        "RESULT scene={scene} width={w} height={h} speed={speed} asm={} frames=8 seconds={secs:.6} fps={:.4} kbps={:.4}",
        cfg!(feature = "asm"),
        8.0 / secs,
        bytes as f64 * 8.0 * 30.0 / 8.0 / 1000.0
    );
}
