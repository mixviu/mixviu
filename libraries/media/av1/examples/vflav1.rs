//! Disposable VFL skip measurement. Single GOP, rav1e 0.8.1 low latency only.
use media_av1::{Encoder, Packet, Settings, Target};
use media_vfl::reader::Reader;
use media_webm::writer::{Writer, Video, av1_config};
use std::{fs::File, io::BufWriter, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Mux {
    writer: Option<Writer<BufWriter<File>>>,
    path: String,
    pending: Option<(Packet, u64)>,
    timeline: Vec<u64>,
    mode: String,
    outputs: u64,
}
impl Mux {
    fn emit(&mut self, next: u64) -> Result<()> {
        let Some((p, slot)) = self.pending.take() else { return Ok(()) };
        let w = self.writer.as_mut().unwrap();
        if self.mode == "vfr" {
            w.packet(1, slot * 1000 / 30, next * 1000 / 30 - slot * 1000 / 30, p.key_frame, &p.data)?;
            self.outputs += 1;
        } else {
            w.packet(1, slot * 1000 / 30, (slot+1)*1000/30-slot*1000/30, p.key_frame, &p.data)?;
            self.outputs += 1;
            if next > slot+1 {
                assert!(!p.key_frame, "cannot re-show displayed keyframe");
                // temporal delimiter, FRAME_HEADER OBU, one byte: show_existing,
                // map index = compressed input order_hint & 3, trailing_one_bit.
                let repeat = [0x12, 0, 0x1a, 1, 0x88 | ((p.pts as u8 & 3) << 4)];
                for i in slot+1..next {
                    w.packet(1,i*1000/30,(i+1)*1000/30-i*1000/30,false,&repeat)?;
                    self.outputs += 1;
                }
            }
        }
        Ok(())
    }
    fn packets(&mut self, packets: Vec<Packet>) -> Result<()> {
        for p in packets {
            let slot = self.timeline[p.pts as usize];
            if self.writer.is_none() {
                self.writer = Some(Writer::new(BufWriter::new(File::create(&self.path)?), Video {width:3840,height:2160,codec_private:av1_config(&p.data)?},None)?);
            }
            self.emit(slot)?;
            self.pending = Some((p,slot));
        }
        Ok(())
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).ok_or("mode full|vfr|cfr-skip")?.clone();
    assert!(["full","vfr","cfr-skip"].contains(&mode.as_str()));
    let mut r = Reader::open(args.get(2).ok_or("VFL directory")?)?;
    assert_eq!((r.width(),r.height(),r.fps()),(3840,2160,30));
    let mut e = Encoder::new(Settings {width:3840,height:2160,fps_num:30,fps_den:1,speed:10,target:Target::Quality(100),key_interval:1_000_000,tile_cols:4,tile_rows:4,threads:16})?;
    let mut mux = Mux {writer:None,path:args.get(3).ok_or("output")?.clone(),pending:None,timeline:vec![],mode:mode.clone(),outputs:0};
    let start = Instant::now();
    let (mut count,mut repeats,mut repeat_submit_seconds,mut change_submit_seconds) = (0u64,0u64,0f64,0f64);
    let mut prepared = None;
    while let Some(f) = r.read()? {
        assert_eq!(f.t_us,f.slot*1_000_000/30);
        count += 1;
        repeats += u64::from(f.repeat);
        if !f.repeat {
            prepared = Some(e.prepare(&media_frame::Bgra::new(3840,2160,f.pixels.to_vec())?.to_yuv420())?);
        }
        if mode != "full" && f.repeat && !(mode == "cfr-skip" && f.slot == 1) { continue; }
        mux.timeline.push(f.slot);
        let t = Instant::now();
        let packets = e.submit(prepared.as_ref().ok_or("repeat before key")?)?;
        if f.repeat { repeat_submit_seconds += t.elapsed().as_secs_f64(); } else { change_submit_seconds += t.elapsed().as_secs_f64(); }
        mux.packets(packets)?;
        if mux.timeline.len() % 100 == 0 { eprintln!("progress {mode} slot={} submitted={} elapsed={:.1}",f.slot,mux.timeline.len(),start.elapsed().as_secs_f64()); }
    }
    mux.packets(e.finish()?)?;
    mux.emit(count)?;
    mux.writer.take().ok_or("empty")?.finish()?;
    let bytes = std::fs::metadata(&mux.path)?.len();
    println!("{{\"mode\":\"{mode}\",\"frames\":{count},\"repeats\":{repeats},\"submitted\":{},\"outputs\":{},\"bytes\":{bytes},\"seconds\":{},\"repeat_submit_seconds\":{repeat_submit_seconds},\"change_submit_seconds\":{change_submit_seconds}}}",mux.timeline.len(),mux.outputs,start.elapsed().as_secs_f64());
    Ok(())
}
fn main() { if let Err(e) = run() { eprintln!("{e}"); std::process::exit(1); } }
