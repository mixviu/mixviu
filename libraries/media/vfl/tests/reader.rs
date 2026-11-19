use media_vfl::{Frame, Pixels, Writer, reader::{Reader, verify}};
use serde_json::{json,Value};
use std::{path::{Path,PathBuf},sync::Arc};
struct Take(PathBuf);
impl Take {
    fn new() -> Self { Self(std::env::temp_dir().join(format!("vfl-reader-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))) }
}
impl Drop for Take {fn drop(&mut self) {let _=std::fs::remove_dir_all(&self.0);}}
fn json_edit(p: &Path, f: impl FnOnce(&mut Value)) {let mut v=serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();f(&mut v);std::fs::write(p,v.to_string()).unwrap();}
fn take(complete:bool) -> (Take,Vec<Vec<u8>>) {
    let t=Take::new(); let mut w=Writer::new(t.0.clone(),4,4,1).unwrap();w.origin(123).unwrap();
    let a=vec![1;64];let mut b=a.clone(); b[20..28].fill(3);b[36..44].fill(9);
    let c=vec![42;64];
    let frames=vec![a.clone(),a.clone(),b.clone(),c.clone(),c.clone(),c.clone(),c.clone(),c.clone(),c.clone(),c.clone(),c.clone(),a.clone()];
    for (slot,p) in [(0,a.clone()),(2,b),(3,c),(11,a)] {w.frame(Frame{slot,pixels:Pixels::Bgra(p.into())}).unwrap();}
    w.frame(Frame{slot:12,pixels:Pixels::Marker(json!({"seq":0,"label":"demo","data":{"x":1},"t_us":100,"frame":0}))}).unwrap();
    w.finish(12,2,123,None,complete).unwrap();(t,frames)
}
#[test]
fn roundtrip_all_kinds_dropped_interrupted_and_seek() {
    for complete in [true,false] {
        let (t,expected)=take(complete);let mut r=Reader::open(&t.0).unwrap();assert_eq!(r.complete,complete);
        for (i,p) in expected.iter().enumerate() {let f=r.read().unwrap().unwrap();assert_eq!(f.slot,i as u64);assert_eq!(f.t_us,i as u64*1_000_000);assert_eq!(f.pixels,p);}
        assert!(r.read().unwrap().is_none());
        for i in [11,0,10,3,2,1] {assert_eq!(r.seek(i).unwrap().pixels,expected[i as usize]);}
        let v=verify(&t.0);assert_eq!(v["ok"],true,"{v}");assert_eq!(v["frames"],12);assert_eq!(v["markers"],1);assert_eq!(v["complete"],complete);
        assert_eq!(v["dropped"],if complete {json!(2)} else {Value::Null});
    }
}
#[test]
fn corruptions_are_rejected_without_panics() {
    for case in 0..10 {
        let (t,_)=take(true);let chunk=t.0.join("chunk-000000.bin");
        match case {
            0=>{let mut b=std::fs::read(&chunk).unwrap();b[18]^=1;std::fs::write(&chunk,b).unwrap();}
            1=>{let mut b=std::fs::read(&chunk).unwrap();b.pop();std::fs::write(&chunk,b).unwrap();}
            2=>json_edit(&t.0.join("manifest.json"),|v|v["version"]=json!(1)),
            3=>{std::fs::rename(&chunk,t.0.join("chunk-000001.bin")).unwrap();}
            4=>json_edit(&t.0.join("complete.json"),|v|v["frames"]=json!(99)),
            5=>{let mut b=std::fs::read(&chunk).unwrap();b[..4].copy_from_slice(&u32::MAX.to_le_bytes());std::fs::write(&chunk,b).unwrap();}
            6=>json_edit(&t.0.join("chunk-000000.json"),|v|v["crc32"]=json!(0)),
            7=>{std::fs::write(t.0.join("frames.jsonl"),"{}").unwrap();}
            8=>{std::fs::write(t.0.join("markers.jsonl"),"{\"seq\":1}\n").unwrap();}
            _=>json_edit(&t.0.join("manifest.json"),|v|v["width"]=json!(u64::MAX)),
        }
        assert_eq!(verify(&t.0)["ok"],false,"case {case}");
    }
}
#[test]
fn v1_without_rectangles_and_missing_recovery_sidecars() {
    let t=Take::new();let mut w=Writer::new(t.0.clone(),2,2,30).unwrap();w.origin(1).unwrap();
    w.frame(Frame{slot:0,pixels:Pixels::Bgra(Arc::from([4;16]))}).unwrap();w.finish(2,0,1,None,true).unwrap();
    for p in ["manifest.json","complete.json"] {json_edit(&t.0.join(p),|v|v["version"]=json!(1));}
    assert_eq!(verify(&t.0)["ok"],true);
    std::fs::remove_file(t.0.join("complete.json")).unwrap();
    for p in ["origin.json","frames.jsonl","markers.jsonl","chunk-000000.json"] {std::fs::remove_file(t.0.join(p)).unwrap();}
    let mut r=Reader::open(&t.0).unwrap();assert!(!r.complete);assert_eq!(r.read().unwrap().unwrap().pixels,[4;16]);assert!(r.read().unwrap().is_some());assert!(r.read().unwrap().is_none());
}
#[test]
#[ignore = "60 second generated 4K30 export measurement on CBC"]
fn bench_export() {
    for busy in [false,true] {
        let t=Take::new();let mut w=Writer::new(t.0.clone(),3840,2160,30).unwrap();w.origin(0).unwrap();
        let mut pixels=vec![0;3840*2160*4];
        for n in 0..1800 {
            if busy {for (i,p) in pixels.iter_mut().enumerate() {*p=((i*37+i/31+n*13)%256) as u8;}}
            else {pixels[n%1024]=(n%256) as u8;}
            w.frame(Frame{slot:n as u64,pixels:Pixels::Bgra(pixels.clone().into())}).unwrap();
        }
        w.finish(1800,0,0,None,true).unwrap();drop(w);
        for _ in 0..3 {
            let out=std::process::Command::new("/usr/bin/time").args(["-v"]).arg(std::env::current_exe().unwrap()).args(["--ignored","--exact","bench_child","--nocapture"]).env("VFL_BENCH_TAKE",&t.0).output().unwrap();
            assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));
            println!("VFL_BENCH busy={busy}\n{}\n{}",String::from_utf8_lossy(&out.stdout),String::from_utf8_lossy(&out.stderr));
        }
    }
}
#[test]
#[ignore = "child process of bench_export"]
fn bench_child() {
    let Some(path)=std::env::var_os("VFL_BENCH_TAKE") else {return};
    use std::io::Write;
    let mut r=Reader::open(path).unwrap();let now=std::time::Instant::now();let mut n=0;
    let mut sink=std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
    while let Some(f)=r.read().unwrap() {sink.write_all(f.pixels).unwrap();n+=1;}
    println!("frames={n} seconds={} fps={}",now.elapsed().as_secs_f64(),n as f64/now.elapsed().as_secs_f64());assert_eq!(n,1800);
}
