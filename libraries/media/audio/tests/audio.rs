use media_audio::{Audio, Error, WavFormat};
use std::io::Cursor;
fn tone(rate: u32, seconds: usize, db: f64, channels: u16) -> Audio {
    Audio::new(
        rate,
        channels,
        (0..rate as usize * seconds)
            .flat_map(|n| {
                let s = 10.0_f64.powf(db / 20.0)
                    * (std::f64::consts::TAU * 1000.0 * n as f64 / rate as f64).sin();
                std::iter::repeat_n(s, channels as usize)
            })
            .collect(),
    )
    .unwrap()
}
#[test]
fn formats_and_malformed_headers() {
    let a = Audio::new(48000,1,vec![0.25]).unwrap();
    let mut b = Cursor::new(Vec::new()); a.write_wav(&mut b,WavFormat::Pcm24).unwrap();
    assert_eq!(Audio::read_wav(Cursor::new(b.into_inner())).unwrap(),a);
    assert!(Audio::new(u32::MAX,2,vec![0.0;2]).unwrap().write_wav(Cursor::new(Vec::new()),WavFormat::Pcm32).is_err());
    for f in [
        WavFormat::Pcm16,
        WavFormat::Pcm24,
        WavFormat::Pcm32,
        WavFormat::Float32,
    ] {
        for c in [1, 2] {
            let a = tone(44100, 1, -20.0, c);
            let mut b = Cursor::new(Vec::new());
            a.write_wav(&mut b, f).unwrap();
            let bytes = b.into_inner();
            let r = Audio::read_wav(Cursor::new(&bytes)).unwrap();
            assert_eq!((r.rate(), r.channels(), r.frames()), (44100, c, 44100));
            assert!(
                r.samples()
                    .iter()
                    .zip(a.samples())
                    .all(|(a, b)| (a - b).abs() < 0.000016)
            );
            for n in 0..bytes.len().min(100) {
                assert!(Audio::read_wav(Cursor::new(&bytes[..n])).is_err());
            }
            for (offset, value) in [(22, 0), (24, 0), (32, 0), (34, 0), (4, 255)] {
                let mut bad = bytes.clone();
                bad[offset] = value;
                assert!(
                    Audio::read_wav(Cursor::new(bad)).is_err(),
                    "offset {offset}"
                );
            }
        }
    }
    // Canonical Google LINEAR16 shape: 44-byte RIFF header, PCM16 mono.
    let mut b = b"RIFF".to_vec();
    b.extend(40u32.to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(48000u32.to_le_bytes());
    b.extend(96000u32.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(4u32.to_le_bytes());
    b.extend([0, 128, 255, 127]);
    assert_eq!(
        Audio::read_wav(Cursor::new(b)).unwrap().samples(),
        [-1.0, 32767.0 / 32768.0]
    );
    assert!(Audio::new(0, 1, vec![]).is_err());
    assert!(Audio::new(48000, 2, vec![0.0]).is_err());
    assert!(Audio::new(48000, 1, vec![f64::NAN]).is_err());
}
#[test]
fn mapping_placement_padding_and_headroom() {
    let a = Audio::new(48000, 1, vec![0.5, -0.5]).unwrap();
    assert_eq!(a.map_channels(2).unwrap().samples(), [0.5, 0.5, -0.5, -0.5]);
    assert_eq!(a.map_channels(2).unwrap().map_channels(1).unwrap(), a);
    let mut out = Audio::silence(48000, 1, 1).unwrap();
    out.place(&a, 2).unwrap();
    out.place(&a, 2).unwrap();
    out.place(&a, 2).unwrap();
    assert_eq!(out.samples(), [0.0, 0.0, 1.5, -1.5]);
    out.resize(3).unwrap();
    out.resize(5).unwrap();
    assert_eq!(out.samples(), [0.0, 0.0, 1.5, 0.0, 0.0]);
    assert!(matches!(
        out.place(&a, usize::MAX),
        Err(Error::SizeOverflow)
    ));
    assert!(out.place(&a.map_channels(2).unwrap(), 0).is_err());
    assert!(Audio::silence(48000,2,0).unwrap().place(&a.map_channels(2).unwrap(),usize::MAX/2).is_err());
}
#[test]
fn tech3341_reference_tone_and_gain() {
    // Tech 3341 stereo 1 kHz at -23 dBFS per channel -> -23 LUFS (±0.1 LU).
    let mut a = tone(48000, 20, -23.0, 2);
    let m = a.measure().unwrap();
    assert!((m.integrated_lufs + 23.0).abs() < 0.1, "{m:?}");
    assert!((m.sample_peak_dbfs + 23.0).abs() < 0.001);
    assert!(m.range_lu < 0.1);
    let r = a.normalise(-14.0, -1.0).unwrap();
    assert!(!r.peak_limited);
    assert!(r.shortfall_lu < 0.01);
    assert!((r.after.integrated_lufs + 14.0).abs() < 0.01);
    assert!(matches!(
        Audio::silence(48000, 2, 48000)
            .unwrap()
            .normalise(-14.0, -1.0),
        Err(Error::Unmeasurable)
    ));
}
#[test]
fn true_peak_not_sample_peak_and_gain_cap() {
    // Quarter-rate sine phase pi/4: intersample peaks exceed samples by ~3 dB.
    let mut a = Audio::new(
        48000,
        1,
        (0..480000)
            .map(|n| {
                0.9 * (std::f64::consts::FRAC_PI_2 * n as f64 + std::f64::consts::FRAC_PI_4).sin()
            })
            .collect(),
    )
    .unwrap();
    let m = a.measure().unwrap();
    assert!(m.true_peak_dbtp - m.sample_peak_dbfs > 2.5, "{m:?}");
    let r = a.normalise(0.0, -1.0).unwrap();
    assert!(r.peak_limited);
    assert!(r.shortfall_lu > 0.1);
    assert!(r.after.true_peak_dbtp <= -0.9999);
}
#[test]
fn resample_lengths_dc_and_tone() {
    for rate in [8000, 16000, 22050, 44100, 48000, 96000, 192000] {
        let a = tone(rate, 1, -20.0, 2).resample_48k().unwrap();
        assert_eq!(a.frames(), 48000);
        let error: f64 = a
            .samples()
            .chunks_exact(2)
            .enumerate()
            .skip(1000)
            .take(46000)
            .map(|(n, s)| {
                (s[0] - 0.1 * (std::f64::consts::TAU * 1000.0 * n as f64 / 48000.0).sin()).powi(2)
            })
            .sum();
        assert!(
            error / 46000.0 < 1e-8,
            "rate {rate} mse {}",
            error / 46000.0
        );
    }
    for n in [1, 10, 1023, 1024, 1025] {
        let a = Audio::new(44100, 1, vec![0.0; n])
            .unwrap()
            .resample_48k()
            .unwrap();
        assert!((a.frames() as isize - (n as f64 * 48000.0 / 44100.0).ceil() as isize).abs() <= 1);
    }
}

#[test]
fn tech3342_range_and_gates() {
    // Equal long plateaux separated by 10 LU: 10th/95th percentiles -> 10 LU.
    let mut a=tone(48000,20,-20.0,2);
    a.place(&tone(48000,20,-30.0,2),a.frames()).unwrap();
    assert!((a.measure().unwrap().range_lu-10.0).abs()<0.2);
    // Silence and a tone 40 LU below programme are removed by the relative gate.
    let mut a=Audio::silence(48000,2,48000*5).unwrap();
    a.place(&tone(48000,20,-23.0,2),48000*5).unwrap();
    a.place(&tone(48000,5,-63.0,2),48000*25).unwrap();
    assert!((a.measure().unwrap().integrated_lufs+23.0).abs()<0.15);
}

#[test]
fn downsample_rejects_aliases() {
    let a=Audio::new(96000,1,(0..96000).map(|n|0.5*(std::f64::consts::TAU*30000.0*n as f64/96000.0).sin()).collect()).unwrap().resample_48k().unwrap();
    let energy=a.samples()[1000..47000].iter().map(|s|s*s).sum::<f64>()/46000.0;
    assert!(energy<1e-12,"alias energy {energy}");
}
