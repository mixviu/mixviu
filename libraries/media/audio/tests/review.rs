use media_audio::{Audio, Error, WavFormat, MAX_RESAMPLED_FRAMES};
use std::io::{Cursor, Read};

fn riff(chunks: &[(&[u8;4], Vec<u8>)]) -> Vec<u8> {
    let mut b = b"RIFF\0\0\0\0WAVE".to_vec();
    for (id, data) in chunks {
        b.extend(*id); b.extend((data.len() as u32).to_le_bytes()); b.extend(data);
        if data.len()%2 != 0 { b.push(0); }
    }
    let n = b.len() as u32 - 8; b[4..8].copy_from_slice(&n.to_le_bytes()); b
}
fn format(tag:u16, bits:u16) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend(tag.to_le_bytes()); f.extend(1u16.to_le_bytes());
    f.extend(48000u32.to_le_bytes()); f.extend((48000u32 * (bits as u32/8)).to_le_bytes());
    f.extend((bits/8).to_le_bytes()); f.extend(bits.to_le_bytes()); f
}
#[test]
fn extensible_narrow_valid_bits_are_refused() {
    for valid in [16u16,24] {
        let mut f = format(0xfffe,32);
        f.extend(22u16.to_le_bytes()); f.extend(valid.to_le_bytes()); f.extend(4u32.to_le_bytes());
        f.extend([1,0,0,0,0,0,16,0,128,0,0,170,0,56,155,113]);
        // Left-aligned +0.25/-0.25 full scale, in a 32-bit container.
        let data = [0x20000000i32, -0x20000000].into_iter().flat_map(i32::to_le_bytes).collect();
        let b = riff(&[(b"fmt ",f),(b"data",data)]);
        let mut h = hound::WavReader::new(Cursor::new(&b)).unwrap();
        assert_eq!(h.spec().bits_per_sample,valid);
        let raw:Vec<_> = h.samples::<i32>().collect();
        println!("hound valid={valid} container=32 raw={raw:?}");
        assert!(matches!(Audio::read_wav(Cursor::new(b)),Err(Error::UnsupportedFormat)));
    }
}
#[test]
fn hostile_riff_and_nonfinite_float_samples() {
    let f = format(1,16); let d=vec![0,0];
    let mut over=riff(&[(b"fmt ",f.clone()),(b"data",d.clone())]);
    over[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    for b in [over,
        riff(&[(b"data",d.clone()),(b"fmt ",f.clone())]),
        riff(&[(b"fmt ",f.clone()),(b"fmt ",f.clone()),(b"data",d.clone())]),
        riff(&[(b"fmt ",f.clone()),(b"data",d.clone()),(b"data",d.clone())]),
        riff(&[(b"fmt ",f)]), riff(&[(b"data",d)])] {
        assert!(matches!(Audio::read_wav(Cursor::new(b)),Err(Error::InvalidHeader)));
    }
    for tag in [2,6,7,0xffff] {
        assert!(matches!(Audio::read_wav(Cursor::new(riff(&[(b"fmt ",format(tag,16)),(b"data",vec![0,0])]))),Err(Error::UnsupportedFormat)));
    }
    for s in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY] {
        let b=riff(&[(b"fmt ",format(3,32)),(b"data",s.to_le_bytes().to_vec())]);
        assert!(matches!(Audio::read_wav(Cursor::new(b)),Err(Error::InvalidAudio)));
    }
    assert!(matches!(Error::from(hound::Error::Unsupported),Error::UnsupportedFormat));
}
#[test]
fn output_range_position_and_early_validation() {
    for f in [WavFormat::Pcm16,WavFormat::Pcm24,WavFormat::Pcm32] {
        for s in [1.0,1.5,-1.001] {
            assert!(matches!(Audio::new(48000,1,vec![s]).unwrap().write_wav(Cursor::new(Vec::new()),f),Err(Error::InvalidAudio)));
        }
    }
    let a=Audio::new(48000,1,vec![1.5]).unwrap(); let mut w=Cursor::new(Vec::new());
    a.write_wav(&mut w,WavFormat::Float32).unwrap();
    assert_eq!(Audio::read_wav(Cursor::new(w.into_inner())).unwrap(),a);
    assert!(Audio::new(48000,1,vec![f64::MAX]).unwrap().write_wav(Cursor::new(Vec::new()),WavFormat::Float32).is_err());
    let mut w=Cursor::new(vec![0x55;20]); w.set_position(10);
    let before=w.clone();
    assert!(matches!(a.write_wav(&mut w,WavFormat::Float32),Err(Error::InvalidWritePosition)));
    assert_eq!(w,before);
    for (rate,channels) in [(0,2),(48000,0),(48000,3)] {
        assert!(matches!(Audio::silence(rate,channels,usize::MAX),Err(Error::InvalidAudio)));
    }
    let mut a=Audio::silence(48000,2,10).unwrap(); let before=a.clone();
    a.place(&Audio::silence(48000,2,0).unwrap(),usize::MAX).unwrap(); assert_eq!(a,before);
}
#[test]
fn reads_and_resample_output_are_bounded() {
    let mut endless=std::io::repeat(0);
    assert!(matches!(Audio::read_wav_limited(&mut endless,32),Err(Error::InputTooLarge{max_bytes:32})));
    let b=riff(&[(b"fmt ",format(1,16)),(b"data",vec![0,0])]);
    assert!(Audio::read_wav_limited(Cursor::new(&b),b.len() as u64).is_ok());
    assert!(matches!(Audio::read_wav_limited(Cursor::new(&b),b.len() as u64-1),Err(Error::InputTooLarge{..})));
    let a=Audio::new(1,1,vec![0.0;MAX_RESAMPLED_FRAMES/48000+1]).unwrap();
    assert!(matches!(a.resample_48k(),Err(Error::OutputTooLong{..})));
    // Read::take does not consume past the single overflow sentinel.
    let mut input=Cursor::new(vec![0;100]); let _=Audio::read_wav_limited(&mut input,5);
    assert_eq!(input.position(),6);
    let mut rest=Vec::new(); input.read_to_end(&mut rest).unwrap(); assert_eq!(rest.len(),94);
}
#[test]
fn dc_gain_and_impulse_position() {
    for rate in [8000,22050,44100,96000,192000] {
        let a=Audio::new(rate,1,vec![0.25;rate as usize]).unwrap().resample_48k().unwrap();
        assert!(a.samples()[2000..46000].iter().all(|s|(s-0.25).abs()<1e-9));
    }
    for rate in [16000,44100,96000] {
        let position=rate as usize/10;
        let mut s=vec![0.0;rate as usize/5];s[position]=0.5;
        let a=Audio::new(rate,1,s).unwrap().resample_48k().unwrap();
        let max=a.samples().iter().enumerate().max_by(|a,b|a.1.abs().total_cmp(&b.1.abs())).unwrap().0;
        assert_eq!(max,4800);
    }
}
#[test]
fn finite_sample_peak_in_true_peak_mode_and_fir_tail() {
    let mut s=vec![0.0;48000];
    s[47996..].copy_from_slice(&[0.63,0.63,-0.63,-0.63]);
    let a=Audio::new(48000,1,s).unwrap();
    let measured=a.measure().unwrap();
    assert!(measured.sample_peak_dbfs.is_finite());
    assert!((measured.sample_peak_dbfs-20.0*0.63_f64.log10()).abs()<1e-12);
    let mut raw=ebur128::EbuR128::new(1,48000,ebur128::Mode::I|ebur128::Mode::LRA|ebur128::Mode::TRUE_PEAK).unwrap();
    raw.add_frames_f64(a.samples()).unwrap();
    let unflushed=raw.true_peak(0).unwrap();
    let integrated=raw.loudness_global().unwrap();
    raw.add_frames_f64(&[0.0;64]).unwrap();
    let flushed=raw.true_peak(0).unwrap();
    println!("FIR tail unflushed={unflushed}, flushed={flushed}");
    assert!(flushed>unflushed+0.05);
    assert!((measured.true_peak_dbtp-20.0*flushed.log10()).abs()<1e-12);
    assert_eq!(measured.integrated_lufs,integrated);
}
