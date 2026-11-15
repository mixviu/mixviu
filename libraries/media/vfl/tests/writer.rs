// SPDX-License-Identifier: MIT OR Apache-2.0
use media_vfl::{RECT_DELTA, Frame, KEY, Pixels, REPEAT, Writer};
use std::{path::PathBuf, sync::Arc};
fn path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "vfl-{}-{}-{}",
        std::process::id(),
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
#[test]
fn key_delta_repeat_round_trip_crc_and_recovery() {
    let path = path("roundtrip");
    let mut writer = Writer::new(path.clone(), 4, 4, 30).unwrap();
    writer.origin(123).unwrap();
    let mut changed = vec![1; 64];
    changed[3] = 2;
    for (slot, pixels) in [vec![1; 64], vec![1; 64], changed.clone()]
        .into_iter()
        .enumerate()
    {
        writer
            .frame(Frame {
                slot: slot as u64,
                pixels: Pixels::Bgra(pixels.into()),
            })
            .unwrap();
    }
    writer.finish(4, 1, 123, None, true).unwrap();
    let data = std::fs::read(path.join("chunk-000000.bin")).unwrap();
    let mut at = 0;
    let mut previous = vec![0; 64];
    for (slot, kind) in [KEY, REPEAT, RECT_DELTA, REPEAT].into_iter().enumerate() {
        let len = u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize;
        assert_eq!(
            crc32fast::hash(&data[at + 8..at + 17 + len]),
            u32::from_le_bytes(data[at + 4..at + 8].try_into().unwrap())
        );
        assert_eq!(
            u64::from_le_bytes(data[at + 8..at + 16].try_into().unwrap()),
            slot as u64
        );
        assert_eq!(data[at + 16], kind);
        let payload = &data[at + 17..at + 17 + len];
        if kind != REPEAT {
            if kind == KEY {
                previous = lz4_flex::block::decompress(payload,64).unwrap();
            } else {
                let values:Vec<usize>=payload[..16].chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap()) as usize).collect();
                let (x,y,w,h)=(values[0],values[1],values[2],values[3]);
                let decoded=lz4_flex::block::decompress(&payload[16..],w*h*4).unwrap();
                for row in 0..h {
                    for (p,d) in previous[((y+row)*4+x)*4..((y+row)*4+x+w)*4].iter_mut().zip(&decoded[row*w*4..(row+1)*w*4]) {*p^=*d;}
                }
            }
        } else {
            assert!(payload.is_empty());
        }
        assert_eq!(
            previous,
            if slot < 2 {
                vec![1; 64]
            } else {
                changed.clone()
            }
        );
        at += 17 + len;
    }
    assert_eq!(at, data.len());
    let complete: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("complete.json")).unwrap()).unwrap();
    assert_eq!(complete["dropped"], 1);
    assert!(Writer::new(path.clone(), 4, 4, 30).is_err());
    std::fs::remove_dir_all(path).unwrap();
}
#[test]
fn rejected_frames_leave_the_log_usable() {
    let path = path("invalid");
    let mut writer = Writer::new(path.clone(), 2, 2, 30).unwrap();
    assert!(
        writer
            .frame(Frame {
                slot: 0,
                pixels: Pixels::Bgra(Arc::from([0; 15]))
            })
            .is_err()
    );
    writer
        .frame(Frame {
            slot: 0,
            pixels: Pixels::Bgra(Arc::from([0; 16])),
        })
        .unwrap();
    writer
        .finish(1, 0, 0, Some("test interruption".into()), false)
        .unwrap();
    assert!(!path.join("complete.json").exists());
    assert!(path.join("interrupted.json").exists());
    std::fs::remove_dir_all(path).unwrap();
}
#[test]
fn rectangle_boundaries_and_whole_frame_fallback() {
    let mut delta=vec![0;64];delta[63]=9;
    let (kind,payload)=media_vfl::compress_delta(&delta,4,4).unwrap();
    assert_eq!(kind,RECT_DELTA);
    assert_eq!(&payload[..16],&[3u32,3,1,1].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>());
    assert_eq!(lz4_flex::block::decompress(&payload[16..],4).unwrap(),vec![0,0,0,9]);
    delta[0]=1;
    assert_eq!(media_vfl::compress_delta(&delta,4,4).unwrap().0,media_vfl::DELTA);
    assert_eq!(media_vfl::compress_delta(&[0;64],4,4).unwrap().0,REPEAT);
    assert!(media_vfl::compress_delta(&[0;63],4,4).is_err());
}
#[test]
#[ignore = "payload measurement on a CBC worker, generated content only"]
fn measure_small_region_delta_4k() {
    let (w, h) = (3840usize, 2160usize);
    let original: Vec<u8> = (0..w * h * 4)
        .map(|i| ((i * 37 + i / 31) % 256) as u8)
        .collect();
    for (rw, rh) in [(1, 1), (64, 64), (256, 256), (1280, 720)] {
        let mut delta = vec![0; original.len()];
        let mut rect = Vec::new();
        for y in 100..100 + rh {
            for x in 100..100 + rw {
                for c in 0..4 {
                    let at = (y * w + x) * 4 + c;
                    let d = original[at] ^ ((x * 13 + y * 17 + c) % 256) as u8;
                    delta[at] = d;
                    rect.push(d);
                }
            }
        }
        let full = lz4_flex::block::compress(&delta).len();
        let roi = 16 + lz4_flex::block::compress(&rect).len();
        let (kind,adaptive)=media_vfl::compress_delta(&delta,w as u32,h as u32).unwrap();
        assert_eq!(kind,RECT_DELTA);
        assert!(adaptive.len()<=full);
        println!(
            "VFL_DELTA_COST region={}x{} whole_payload={} rectangle_payload_with_coordinates={} saving={} raw_frame={}",
            rw,
            rh,
            full,
            roi,
            full - roi,
            original.len()
        );
        println!("VFL_ADAPTIVE_DELTA region={}x{} kind={} payload={}",rw,rh,kind,adaptive.len());
    }
}
