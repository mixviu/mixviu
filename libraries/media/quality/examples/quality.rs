use media_quality::{Sequence, y4m::Reader};
use std::io::{self, BufRead, BufReader, Write};
fn open(path: &str) -> io::Result<Box<dyn BufRead>> {
    if path == "-" { Ok(Box::new(BufReader::new(io::stdin()))) }
    else { Ok(Box::new(BufReader::new(std::fs::File::open(path)?))) }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    compare(&args, &mut io::BufWriter::new(io::stdout().lock()))
}
fn compare(args: &[String], out: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {
    let (a,b) = match args {
        [b] => ("-",b.as_str()), [a,b] if a != "-" || b != "-" => (a.as_str(),b.as_str()),
        _ => return Err("usage: quality [first.y4m|-] second.y4m".into()),
    };
    let (mut a,mut b) = (Reader::new(open(a)?)?,Reader::new(open(b)?)?);
    let mut seq = Sequence::default();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_and_stream_lengths() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.y4m");
        let mut bytes = b"YUV4MPEG2 W16 H16 Ip C420\nFRAME\n".to_vec();
        bytes.extend([80u8;384]);
        std::fs::write(&path,&bytes).unwrap();
        let args = [path.to_str().unwrap().to_owned(),path.to_str().unwrap().to_owned()];
        let mut out = Vec::new();
        compare(&args,&mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("n:1 Y:1.000000 U:1.000000 V:1.000000 All:1.000000"));
        assert!(text.contains("psnr_avg:inf"));
        assert!(text.contains("average:inf min:inf"));
        assert_eq!(text.lines().count(),3);
        let other = dir.path().join("b.y4m");
        std::fs::write(&other,b"YUV4MPEG2 W16 H16 Ip C420\n").unwrap();
        assert!(compare(&[args[0].clone(),other.to_str().unwrap().to_owned()],&mut Vec::new()).is_err());
        assert!(compare(&[],&mut Vec::new()).is_err());
        assert!(compare(&["-".into(),"-".into()],&mut Vec::new()).is_err());
    }
}
