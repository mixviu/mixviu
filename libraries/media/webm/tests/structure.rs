use media_webm::{Demuxer, Error, Video, Writer, av1_config};
use std::io::Cursor;

fn writer() -> Writer<Cursor<Vec<u8>>> {
    Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width: 16,
            height: 16,
            codec_private: vec![0x81, 0, 12, 0],
        },
        None,
    )
    .unwrap()
}
#[test]
fn cluster_boundaries_and_cue_seek() {
    let mut w = writer();
    for (t, key) in [
        (0, true),
        (4999, false),
        (5000, false),
        (6000, true),
        (11000, false),
        (11001, true),
    ] {
        w.packet(1, t, 1, key, &[42]).unwrap();
    }
    let b = w.finish().unwrap().into_inner();
    assert_eq!(
        b.windows(4)
            .filter(|x| *x == [0x1f, 0x43, 0xb6, 0x75])
            .count(),
        5
    );
    let mut d = Demuxer::new(&b).unwrap();
    assert_eq!(d.cues.len(), 3);
    assert_eq!(
        d.packets().iter().map(|p| p.timestamp).collect::<Vec<_>>(),
        [0, 4999, 5000, 6000, 11000, 11001]
    );
    for (t, want) in [
        (0, 0),
        (5999, 0),
        (6000, 6000),
        (11000, 6000),
        (11001, 11001),
        (u64::MAX, 11001),
    ] {
        assert_eq!(d.seek(t), Some(want));
        let p = d.next_packet().unwrap();
        assert_eq!(p.timestamp, want as i64);
        assert!(p.key_frame);
    }
}
#[test]
fn unknown_ids_and_size_errors() {
    let mut w = writer();
    w.packet(1, 0, 1, true, &[42]).unwrap();
    let b = w.finish().unwrap().into_inner();
    // The writer's reserved Void is an unknown ID to the reader and is skipped.
    assert!(Demuxer::new(&b).is_ok());
    let mut hostile = b.clone();
    // EBML header size with all payload bits set except the last bit: huge finite size.
    hostile[4..12].copy_from_slice(&[1, 255, 255, 255, 255, 255, 255, 254]);
    assert!(matches!(
        Demuxer::new(&hostile),
        Err(Error::Truncated) | Err(Error::SizeOverflow)
    ));
    let mut hostile = b.clone();
    hostile[4] = 0;
    assert!(matches!(Demuxer::new(&hostile), Err(Error::SizeOverflow)));
    // Unknown-sized Segment is the supported streaming-input fallback.
    let mut streaming = b.clone();
    let seg = streaming
        .windows(4)
        .position(|b| b == [0x18, 0x53, 0x80, 0x67])
        .unwrap();
    streaming[seg + 4..seg + 12].copy_from_slice(&[1, 255, 255, 255, 255, 255, 255, 255]);
    assert!(Demuxer::new(&streaming).is_ok());
}
#[test]
fn invalid_calls_and_sequence_headers() {
    assert!(writer().finish().is_err());
    let mut w = writer();
    assert!(w.packet(1, 0, 1, false, &[42]).is_err());
    let mut w = writer();
    assert!(w.packet(1, i64::MAX as u64 + 1, 0, true, &[42]).is_err());
    let mut w = writer();
    w.packet(1, 100, 1, true, &[42]).unwrap();
    assert!(w.packet(1, 99, 1, false, &[42]).is_err());
    assert!(w.finish().is_err());
    let mut w = writer();
    assert!(w.packet(2, 0, 1, true, &[42]).is_err());
    let mut w = writer();
    assert!(w.packet(1, u64::MAX, 1, true, &[42]).is_err());
    for b in [&[][..], &[0x0a], &[0x0a, 0xff], &[0x0a, 1, 0], &[0xff]] {
        assert!(av1_config(b).is_err());
    }
}

fn el(id: u32, b: &[u8]) -> Vec<u8> {
    let bytes = id.to_be_bytes();
    let mut out = bytes[bytes.iter().position(|x| *x != 0).unwrap()..].to_vec();
    out.extend(((1u64 << 56) | b.len() as u64).to_be_bytes());
    out.extend(b);
    out
}
fn u(id: u32, n: u64) -> Vec<u8> {
    el(id, &n.to_be_bytes())
}
fn file(body: &[u8]) -> Vec<u8> {
    let mut b = el(0x1a45dfa3, &el(0x4282, b"webm"));
    b.extend(el(0xec, &[]));
    b.extend(el(0x18538067, body));
    b
}
fn track(n: u64, kind: u64) -> Vec<u8> {
    el(
        0xae,
        &[u(0xd7, n), u(0x83, kind), el(0x86, b"V_AV1")].concat(),
    )
}
#[test]
fn missing_timestamp_bad_cue_and_unknown_cluster() {
    let tracks = el(0x1654ae6b, &track(1, 1));
    let block = el(0xa3, &[0x81, 0, 0, 0x80, 42]);
    let missing = file(&[tracks.clone(), el(0x1f43b675, &block)].concat());
    assert!(matches!(
        Demuxer::new(&missing),
        Err(Error::Invalid("Cluster missing Timestamp"))
    ));
    let cluster = el(0x1f43b675, &[el(0xe7, &[]), block].concat());
    let cue = el(
        0x1c53bb6b,
        &el(
            0xbb,
            &[el(0xb3, &[]), el(0xb7, &[u(0xf7, 1), u(0xf1, 0)].concat())].concat(),
        ),
    );
    let bytes = file(&[tracks.clone(), cluster.clone(), cue].concat());
    let d = Demuxer::new(&bytes).unwrap();
    assert_eq!(d.packets()[0].timestamp, 0);
    assert!(d.cues.is_empty());
    let unknown = file(&[tracks, vec![0x1f, 0x43, 0xb6, 0x75, 0xff]].concat());
    assert!(matches!(
        Demuxer::new(&unknown),
        Err(Error::Invalid("unknown size only supported for Segment"))
    ));
}
#[test]
fn hostile_many_tracks_blocks_and_cues_refused_within_bound() {
    const N: u64 = 20_000;
    let tracks = el(
        0x1654ae6b,
        &(1..=N)
            .flat_map(|n| track(n, if n == N { 1 } else { 2 }))
            .collect::<Vec<_>>(),
    );
    let mut blocks = el(0xe7, &[]);
    for i in 0..N * 2 {
        let mut b = ((1u32 << 21) | N as u32).to_be_bytes()[1..].to_vec();
        b.extend([0, 0, if i == N * 2 - 1 { 0x80 } else { 0 }, 42]);
        blocks.extend(el(0xa3, &b));
    }
    let cue = el(
        0xbb,
        &[
            el(0xb3, &[]),
            el(0xb7, &[u(0xf7, N), u(0xf1, tracks.len() as u64)].concat()),
        ]
        .concat(),
    );
    let cues = el(0x1c53bb6b, &cue.repeat(N as usize));
    let bytes = file(&[tracks, el(0x1f43b675, &blocks), cues].concat());
    let start = std::time::Instant::now();
    assert!(Demuxer::new(&bytes).is_err());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "indexed open exceeded five-second bound"
    );
}

#[test]
fn depth_and_codec_limits() {
    let mut nested = Vec::new();
    for _ in 0..10 {
        nested = el(0xae, &nested);
    }
    assert!(matches!(
        Demuxer::new(&file(&nested)),
        Err(Error::Invalid("nesting depth limit"))
    ));
    let bad = el(
        0x1654ae6b,
        &el(0xae, &[u(0xd7, 1), u(0x83, 1), el(0x86, b"V_VP9")].concat()),
    );
    assert!(
        matches!(Demuxer::new(&file(&bad)),Err(Error::UnsupportedCodec(name)) if name == "V_VP9")
    );
}

#[test]
fn srgb_identity_sequence_is_refused() {
    // Reduced still Main sequence through color_description: 1/13/0. This
    // branch omits color_range/chroma_sample_position, so no such bits follow.
    let fields = [
        (0, 3),
        (1, 1),
        (1, 1),
        (0, 5),
        (0, 4),
        (0, 4),
        (0, 1),
        (0, 1),
        (0, 3),
        (0, 3),
        (0, 1),
        (0, 1),
        (1, 1),
        (1, 8),
        (13, 8),
        (0, 8),
    ];
    let mut bits = Vec::new();
    for (v, n) in fields {
        for i in (0..n).rev() {
            bits.push(((v >> i) & 1) as u8);
        }
    }
    let mut bytes = vec![0u8; bits.len().div_ceil(8)];
    for (i, b) in bits.into_iter().enumerate() {
        bytes[i / 8] |= b << (7 - i % 8);
    }
    let mut obu = vec![0x0a, bytes.len() as u8];
    obu.extend(bytes);
    assert!(matches!(
        av1_config(&obu),
        Err(Error::Invalid("sRGB identity colour is not Main 4:2:0"))
    ));
}
