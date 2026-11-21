pub fn make(name: &str, rate: i32) -> Box<dyn FnMut(&[f32]) -> Vec<u8>> {
    macro_rules! family {
        ($m:ident) => {{
            let mut e = $m::OpusEncoder::new(48000,2,$m::Application::Audio).unwrap();
            e.bitrate_bps=rate;
            Box::new(move |p:&[f32]| {let mut b=vec![0;4000];let n=e.encode(p,960,&mut b).unwrap();b.truncate(n);b})
        }};
    }
    macro_rules! recovered {
        ($m:ident) => {{
            let mut e=$m::Encoder::new(48000,$m::Channels::Stereo,$m::Application::Audio).unwrap();
            e.set_bitrate($m::Bitrate::Bits(rate)).unwrap();
            Box::new(move |p:&[f32]| {let mut b=vec![0;4000];let n=e.encode_float(p,&mut b).unwrap();b.truncate(n);b})
        }};
    }
    match name {
        "rusty-opus"=>family!(rusty_opus), "opus-rs"=>family!(opus_rs),
        "opus-pure"=>family!(opus_pure), "cranpose-opus-pure"=>family!(cranpose_opus_pure),
        "hiraku-opus"=>family!(hiraku_opus), "ps-rusty-opus"=>family!(ps_rusty_opus),
        "mousiki"=>recovered!(mousiki), "oporus"=>recovered!(oporus),
        "ruopus"=> {let mut e=ruopus::OpusEncoder::new(2);e.set_bitrate(Some(rate as u32));Box::new(move|p|e.encode_auto(p,4000).unwrap())},
        "opus-wave"=> {let mut e=opus_wave::OpusEncoder::new(opus_wave::SampleRate::Hz48000,opus_wave::Channels::Stereo,opus_wave::Application::Audio).unwrap();e.set_bitrate(opus_wave::Bitrate::BitsPerSecond(rate));Box::new(move|p|{let mut b=vec![0;4000];let n=e.encode_float(p,960,&mut b,4000).unwrap();b.truncate(n as usize);b})},
        "oxideav-opus"=> {let mut e=oxideav_opus::OpusEncoder::new(2,oxideav_opus::Application::Audio,rate as u32).unwrap();Box::new(move|p|{let pcm:Vec<i16>=p.iter().map(|&v|(v*32768.).round().clamp(-32768.,32767.) as i16).collect();e.encode_frame(&pcm).unwrap()})},
        _=>panic!("encoder unavailable")
    }
}

