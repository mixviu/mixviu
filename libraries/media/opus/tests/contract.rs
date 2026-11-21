use media_opus::{Channels, Decoder, Encoder, FRAME_SAMPLES, OpusDecoder, OpusEncoder};

#[test]
fn header_flush_and_explicit_loss() {
    for channels in [Channels::Mono, Channels::Stereo] {
        let mut encoder = OpusEncoder::new(channels, 96_000).unwrap();
        let head = encoder.head().clone();
        assert_eq!(head.bytes().len(), 19);
        assert_eq!(&head.bytes()[..8], b"OpusHead");
        assert_eq!(head.channels as usize, channels.count());
        assert_eq!(head.mapping_family, 0);
        assert_eq!(head.stream_count, 1);
        assert_eq!(head.coupled_count, u8::from(channels == Channels::Stereo));
        assert_eq!(head.bytes(), [79,112,117,115,72,101,97,100,1,channels.count() as u8,56,1,128,187,0,0,0,0,0]);
        let mut decoder = OpusDecoder::new(channels).unwrap();
        let mut samples = 0;
        for _ in 0..5 {
            let packet = encoder
                .encode(&vec![0.1; FRAME_SAMPLES * channels.count()])
                .unwrap();
            let output = decoder.decode(Some(&packet.data)).unwrap();
            assert!(!output.packet_lost);
            assert_eq!(output.channels, channels);
            assert_eq!(output.pcm.len(), FRAME_SAMPLES * channels.count());
            samples += output.pcm.len() / channels.count();
        }
        let tail = encoder.finish().unwrap().unwrap();
        samples += decoder.decode(Some(&tail.data)).unwrap().pcm.len() / channels.count();
        assert_eq!(
            samples - usize::from(head.pre_skip) - usize::from(tail.discard_padding),
            5 * FRAME_SAMPLES
        );
        assert!(encoder.finish().unwrap().is_none());
        assert!(
            encoder
                .encode(&vec![0.; FRAME_SAMPLES * channels.count()])
                .is_err()
        );
        let lost = decoder.decode(None).unwrap();
        assert!(lost.packet_lost);
        assert_eq!(lost.pcm.len(), FRAME_SAMPLES * channels.count());
        assert!(decoder.decode(Some(&[])).is_err());
        let mut resumed = OpusEncoder::new(channels, 96_000).unwrap();
        let packet = resumed.encode(&vec![0.1; FRAME_SAMPLES * channels.count()]).unwrap();
        let output = decoder.decode(Some(&packet.data)).unwrap();
        assert!(!output.packet_lost);
        assert_eq!(output.pcm.len(), FRAME_SAMPLES * channels.count());
    }
}

#[test]
fn bitrate_controls_mean_payload_size() {
    let mut low = OpusEncoder::new(Channels::Stereo, 64_000).unwrap();
    let mut high = OpusEncoder::new(Channels::Stereo, 128_000).unwrap();
    let mut seed = 0x5749_132au32;
    let mut sizes = [0usize; 2];
    for _ in 0..100 {
        let frame: Vec<f32> = (0..FRAME_SAMPLES * 2).map(|_| {
            seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
            (seed as i32 as f32 / i32::MAX as f32) * 0.35
        }).collect();
        sizes[0] += low.encode(&frame).unwrap().data.len();
        sizes[1] += high.encode(&frame).unwrap().data.len();
    }
    assert!(sizes[1] > sizes[0] * 3 / 2, "128k payload {} must clearly exceed 64k {}", sizes[1], sizes[0]);
}

#[test]
fn damaged_real_packets_with_interleaved_loss() {
    for channels in [Channels::Mono, Channels::Stereo] {
        let mut encoder = OpusEncoder::new(channels, 96_000).unwrap();
        let pcm: Vec<f32> = (0..FRAME_SAMPLES * channels.count()).map(|i| (i as f32 * 0.037).sin() * 0.3).collect();
        let prime = encoder.encode(&pcm).unwrap();
        let packet = encoder.encode(&pcm).unwrap();
        for cut in [1, packet.data.len()/2, packet.data.len()-1] {
            let mut decoder = OpusDecoder::new(channels).unwrap();
            decoder.decode(Some(&prime.data)).unwrap();
            assert!(decoder.decode(None).unwrap().packet_lost);
            if let Ok(out) = decoder.decode(Some(&packet.data[..cut])) {
                assert!(out.pcm.iter().all(|v| v.is_finite()));
            }
            decoder.decode(None).unwrap();
            let out = decoder.decode(Some(&packet.data)).unwrap();
            assert!(!out.packet_lost);
            assert_eq!(out.pcm.len(), FRAME_SAMPLES * channels.count());
        }
        for index in [0, packet.data.len()/2, packet.data.len()-1] {
            let mut decoder = OpusDecoder::new(channels).unwrap();
            decoder.decode(Some(&prime.data)).unwrap();
            let mut damaged = packet.data.clone();
            damaged[index] ^= 0x80;
            if let Ok(out) = decoder.decode(Some(&damaged)) {
                assert!(out.pcm.iter().all(|v| v.is_finite()));
            }
            assert!(decoder.decode(None).unwrap().packet_lost);
            assert!(!decoder.decode(Some(&packet.data)).unwrap().packet_lost);
        }
    }
}

#[test]
fn invalid_inputs_and_corrupt_packets_do_not_panic() {
    assert!(OpusEncoder::new(Channels::Mono, 0).is_err());
    assert!(OpusEncoder::new(Channels::Stereo, u32::MAX).is_err());
    let mut encoder = OpusEncoder::new(Channels::Mono, 64_000).unwrap();
    assert!(encoder.encode(&[0.; 959]).is_err());
    assert!(encoder.encode(&[f32::NAN; 960]).is_err());
    assert!(encoder.finish().unwrap().is_none());
    for channels in [Channels::Mono, Channels::Stereo] {
        let mut decoder = OpusDecoder::new(channels).unwrap();
        assert!(decoder.decode(Some(&[255, 63])).is_err());
        assert!(decoder.decode(Some(&vec![0; 65_536])).is_err());
        let mut seed = 0x1234_5678u32;
        for len in 1..128 {
            let mut bytes = vec![0; len];
            for b in &mut bytes {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                *b = seed as u8;
            }
            // Some random packets are legal. Both a finite decode and an error
            // are valid; unwinding through this public API is not.
            if let Ok(out) = decoder.decode(Some(&bytes)) {
                assert!(out.pcm.iter().all(|v| v.is_finite()));
                assert!(out.pcm.len() <= 5760 * channels.count());
            }
        }
    }
}
