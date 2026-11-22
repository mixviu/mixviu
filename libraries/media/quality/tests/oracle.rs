use media_frame::Yuv420;
use media_quality::{Decibels, psnr, ssim};
use std::{io::Write, process::Command};
fn command(args: &[&str]) -> Vec<u8> {
    let o = Command::new("ffmpeg").args(args).output().expect("ffmpeg was probed");
    assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr)); o.stdout
}
fn available() -> bool {
    let probe = Command::new("ffmpeg").args(["-v","error","-filters"]).output();
    let ok = probe.as_ref().is_ok_and(|o| o.status.success() && [" ssim "," psnr "].iter().all(|s| String::from_utf8_lossy(&o.stdout).contains(s)));
    if !ok {
        eprintln!("SKIP: ffmpeg with ssim and psnr filters unavailable");
        assert_ne!(std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(),Ok("1"),"ffmpeg ssim/psnr required");
    }
    ok
}
fn value(line:&str,key:&str) -> f64 {
    line.split_whitespace().find_map(|x| x.strip_prefix(&format!("{key}:"))).unwrap().parse().unwrap()
}
#[test]
fn ffmpeg_generated_corpus() {
    if !available() { return; }
    eprintln!("{}",String::from_utf8_lossy(&command(&["-version"])));
    let dir = tempfile::tempdir().unwrap();
    let mut maxima = [[0.0f64;4];2];
    // Self-contained public corpus: moving detail plus flat deck-like panels.
    // No private captures or separately provisioned fixtures are required.
    for (w,h,source) in [(96usize,64usize,"testsrc2=size=96x64:rate=30"),(35,33,"nullsrc=size=35x33:rate=30,format=yuv420p,geq=lum='mod(X*7+Y*13+N*11,256)':cb='mod(X*17+N,256)':cr='mod(Y*19+N,256)'"),(320,180,"color=c=navy:size=320x180:rate=30,drawbox=x=20:y=20:w=280:h=40:color=white:t=fill,drawbox=x=20:y=80:w=130:h=70:color=yellow:t=fill")] {
        let raw = command(&["-v","error","-f","lavfi","-i",source,"-frames:v","6","-pix_fmt","yuv420p","-f","rawvideo","-"]);
        let n = w*h+2*w.div_ceil(2)*h.div_ceil(2);
        assert_eq!(raw.len(),n*6);
        let mut other = raw.clone();
        for (i,x) in other.iter_mut().enumerate() { if i%7==0 { *x=x.saturating_add((i%13) as u8); } }
        let a = dir.path().join("a.yuv"); let b = dir.path().join("b.yuv");
        std::fs::File::create(&a).unwrap().write_all(&raw).unwrap();
        std::fs::write(&b,&other).unwrap();
        let size = format!("{w}x{h}");
        for (metric,keys) in [("ssim",["Y","U","V","All"]),("psnr",["psnr_y","psnr_u","psnr_v","psnr_avg"])] {
            let stats = dir.path().join("stats");
            let filter = format!("{metric}=stats_file={}",stats.display());
            command(&["-v","error","-f","rawvideo","-pixel_format","yuv420p","-video_size",&size,"-i",a.to_str().unwrap(),"-f","rawvideo","-pixel_format","yuv420p","-video_size",&size,"-i",b.to_str().unwrap(),"-lavfi",&filter,"-f","null","-"]);
            let lines = std::fs::read_to_string(stats).unwrap(); assert_eq!(lines.lines().count(),6);
            for (i,line) in lines.lines().enumerate() {
                let frame = |data:&[u8]| -> Yuv420 { let y=w*h; let c=w.div_ceil(2)*h.div_ceil(2); let d=&data[i*n..(i+1)*n]; Yuv420 { width:w,height:h,y:d[..y].to_vec(),u:d[y..y+c].to_vec(),v:d[y+c..].to_vec() } };
                let (a,b)=(frame(&raw),frame(&other));
                let ours = if metric=="ssim" { let s=ssim(&a,&b).unwrap(); [s.y,s.u,s.v,s.all] } else { let p=psnr(&a,&b).unwrap(); [p.y,p.u,p.v,p.all].map(|d| match d { Decibels::Finite(v)=>v,Decibels::Infinite=>f64::INFINITY }) };
                for j in 0..4 {
                    let oracle = value(line,keys[j]);
                    let diff = if oracle==ours[j] {0.0} else {(oracle-ours[j]).abs()};
                    let row = usize::from(metric=="psnr"); maxima[row][j]=maxima[row][j].max(diff);
                    let precision = if metric=="ssim" {6} else {2};
                    assert!(diff<=1e-6 || format!("{:.*}",precision,ours[j])==format!("{:.*}",precision,oracle),"{metric} {} frame {i}: ours={} ffmpeg={oracle}",keys[j],ours[j]);
                }
            }
        }
    }
    println!("oracle max abs [Y,U,V,All] SSIM={:?} PSNR_dB={:?} (PSNR oracle prints 2 decimals)",maxima[0],maxima[1]);
}
