//! Worker-only fixture generator/comparator. Does not launch external tools.
use media_frame::Bgra;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let dir = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(dir).unwrap();
    let mut pixels=Vec::new();
    for y in 0..64 { for x in 0..128 {
        let p=if y<32 { [[0,0,0,255],[255,255,255,255],[0,0,255,255],[0,255,0,255],[255,0,0,255]][x/26 % 5] }
        else { [((x*19+y*7)%256) as u8,((x*3+y*17)%256) as u8,((x*13+y*11)%256) as u8,255] };
        pixels.extend_from_slice(&p);
    }}
    let f=Bgra::new(128,64,pixels).unwrap(); let yuv=f.to_yuv420();
    std::fs::write(dir.join("source.bgra"),f.pixels()).unwrap();
    let raw=[yuv.y,yuv.u,yuv.v].concat();
    std::fs::write(dir.join("source.yuv"),&raw).unwrap();
    if args.len()>2 {
        let other=std::fs::read(dir.join("oracle.yuv")).unwrap(); assert_eq!(raw.len(),other.len());
        let mut offset=0;
        for (name,n) in [("Y",8192),("U",2048),("V",2048)] {
            let diffs:Vec<_>=raw[offset..offset+n].iter().zip(&other[offset..offset+n]).map(|(a,b)|a.abs_diff(*b) as usize).collect();
            let max=*diffs.iter().max().unwrap(); let mean=diffs.iter().sum::<usize>() as f64/n as f64;
            println!("{name}: max={max} mean={mean:.6}"); assert!(max<=2 && mean<=0.6); offset+=n;
        }
    }
}
