use media_quality::{Sequence, y4m::Reader};
use std::io::{self, BufRead, BufReader, Write};
fn open(path: &str) -> io::Result<Box<dyn BufRead>> {
    if path == "-" { Ok(Box::new(BufReader::new(io::stdin()))) }
    else { Ok(Box::new(BufReader::new(std::fs::File::open(path)?))) }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (a,b) = match args.as_slice() {
        [b] => ("-",b.as_str()), [a,b] if a != "-" || b != "-" => (a.as_str(),b.as_str()),
        _ => return Err("usage: quality [first.y4m|-] second.y4m".into()),
    };
    let (mut a,mut b) = (Reader::new(open(a)?)?,Reader::new(open(b)?)?);
    let mut seq = Sequence::default();
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut n = 0u64;
    loop {
        match (a.next_frame()?,b.next_frame()?) {
            (None,None) => break,
            (Some(a),Some(b)) => {
                let (s,p) = seq.update(&a,&b)?; n += 1;
                writeln!(out,"n:{n} Y:{:.6} U:{:.6} V:{:.6} All:{:.6} mse_avg:{:.2} mse_y:{:.2} mse_u:{:.2} mse_v:{:.2} psnr_avg:{} psnr_y:{} psnr_u:{} psnr_v:{}",s.y,s.u,s.v,s.all,p.mse_all,p.mse_y,p.mse_u,p.mse_v,p.all,p.y,p.u,p.v)?;
            }
            _ => return Err("stream frame counts differ".into()),
        }
    }
    if let Some(s) = seq.summary() {
        writeln!(out,"SSIM Y:{:.6} U:{:.6} V:{:.6} All:{:.6} min:{:.6}",s.mean_ssim.y,s.mean_ssim.u,s.mean_ssim.v,s.mean_ssim.all,s.min_ssim.all)?;
        writeln!(out,"PSNR y:{} u:{} v:{} average:{} min:{}",s.mean_psnr.y,s.mean_psnr.u,s.mean_psnr.v,s.mean_psnr.all,s.min_psnr.all)?;
    } else { return Err("empty streams".into()); }
    out.flush()?;
    Ok(())
}
