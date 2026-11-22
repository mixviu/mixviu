use media_frame::Yuv420;
use media_quality::{Decibels, Error, Sequence, psnr, ssim, y4m::Reader};
use std::io::Cursor;
fn frame(w:usize,h:usize,value:u8) -> Yuv420 {
    Yuv420 { width:w,height:h,y:vec![value;w*h],u:vec![value;w.div_ceil(2)*h.div_ceil(2)],v:vec![value;w.div_ceil(2)*h.div_ceil(2)] }
}
#[test]
fn identity_offset_and_sequence() {
    let a = frame(32,32,80);
    let b = frame(32,32,90);
    assert_eq!(ssim(&a,&a).unwrap().all,1.0);
    let p = psnr(&a,&a).unwrap();
    assert_eq!([p.y,p.u,p.v,p.all],[Decibels::Infinite;4]);
    let p = psnr(&a,&b).unwrap();
    assert_eq!([p.mse_y,p.mse_u,p.mse_v,p.mse_all],[100.0;4]);
    assert_eq!(p.all,Decibels::Finite(20.0*(255.0f64/10.0).log10()));
    let s = ssim(&a,&b).unwrap();
    let expected = ((2*5120*5760+416) as f32 / (5120*5120+5760*5760+416) as f32) as f64;
    assert!((s.all-expected).abs()<1e-7);
    let mut seq = Sequence::default();
    assert!(seq.summary().is_none());
    seq.update(&a,&a).unwrap(); seq.update(&a,&b).unwrap();
    let sum = seq.summary().unwrap();
    assert_eq!(sum.frames,2);
    assert!((sum.mean_ssim.all-(1.0+s.all)/2.0).abs()<1e-12);
    assert_eq!(sum.min_ssim,s);
    assert_eq!(sum.mean_psnr.mse_all,50.0);
    assert_eq!(sum.min_psnr,p);
}
#[test]
fn invalid_and_small() {
    let a = frame(32,32,0);
    assert_eq!(psnr(&a,&frame(33,32,0)),Err(Error::SizeMismatch));
    let mut b = a.clone(); b.u.pop();
    assert_eq!(ssim(&a,&b),Err(Error::InvalidFormat));
    assert_eq!(psnr(&a,&b),Err(Error::InvalidFormat));
    let mut b = a.clone(); b.width = usize::MAX; b.height = usize::MAX;
    assert_eq!(psnr(&b,&b),Err(Error::InvalidFormat));
    let b = frame(1,1,0);
    assert_eq!(ssim(&b,&b),Err(Error::NoSsimWindow));
    assert_eq!(psnr(&b,&b).unwrap().all,Decibels::Infinite);
    let mut seq = Sequence::default(); assert!(seq.update(&a,&b).is_err());
    assert!(seq.summary().is_none());
}
#[test]
fn odd_edges_and_weighting() {
    let a = frame(35,33,0); let mut b = a.clone();
    b.y[34] = 255; // ignored trailing SSIM column, counted by PSNR
    assert_eq!(ssim(&a,&b).unwrap().y,1.0);
    let p = psnr(&a,&b).unwrap();
    assert_eq!(p.mse_y,65025.0/(35.0*33.0));
    assert!((p.mse_all-65025.0/(35.0*33.0+2.0*18.0*17.0)).abs()<1e-12);
    b.u.fill(80);
    let s = ssim(&a,&b).unwrap();
    assert!((s.all-(s.y*1155.0+s.u*306.0+s.v*306.0)/1767.0).abs()<1e-12);
}
#[test]
fn y4m_streaming_and_errors() {
    let mut bytes = b"YUV4MPEG2 W3 H1 F30:1 Ip C420jpeg\nFRAME tag\n".to_vec();
    bytes.extend([1,2,3,4,5,6,7]);
    let mut r = Reader::new(Cursor::new(&bytes)).unwrap();
    let f = r.next_frame().unwrap().unwrap();
    assert_eq!(f.y,[1,2,3]); assert_eq!(f.u,[4,5]); assert_eq!(f.v,[6,7]);
    assert!(r.next_frame().unwrap().is_none());
    bytes.pop(); assert!(Reader::new(Cursor::new(bytes)).unwrap().next_frame().is_err());
    for h in ["YUV4MPEG2 W3 H1 C420p10\n","YUV4MPEG2 W0 H1\n","bad\n","YUV4MPEG2 W3 H1 C444\n"] {
        assert!(Reader::new(Cursor::new(h)).is_err());
    }
}
#[test]
#[ignore = "explicit release throughput measurement"]
fn throughput_4k() {
    let mut a = frame(3840,2160,80); let mut b = a.clone();
    for (i,x) in a.y.iter_mut().enumerate() { *x = (i*31+i/3840) as u8; }
    b.y.clone_from(&a.y); for x in b.y.iter_mut().step_by(7) { *x = x.wrapping_add(3); }
    let start = std::time::Instant::now();
    for _ in 0..120 { std::hint::black_box(ssim(std::hint::black_box(&a),std::hint::black_box(&b)).unwrap()); }
    println!("4K SSIM frames=120 seconds={:.6} fps={:.3}",start.elapsed().as_secs_f64(),120.0/start.elapsed().as_secs_f64());
}
