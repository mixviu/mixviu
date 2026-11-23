//! CBC benchmark: av1dec FILE.webm [THREADS]. Decode and copy every plane.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let bytes = std::fs::read(args.next().ok_or("input required")?)?;
    let threads = args.next().unwrap_or("4".into()).parse()?;
    let demux = media_webm::Demuxer::new(&bytes)?;
    let track = demux.tracks.iter().find(|t| t.codec == "V_AV1").ok_or("no video")?;
    let mut decoder = media_av1::Decoder::new(threads,500_000)?;
    let start = std::time::Instant::now();
    let mut frames = 0;
    let mut emit = |f| { std::hint::black_box(f); frames += 1; Ok(()) };
    for p in demux.packets().iter().filter(|p| p.track == track.number) { decoder.submit(p.data,p.timestamp,&mut emit)?; }
    decoder.finish(&mut emit)?;
    let seconds = start.elapsed().as_secs_f64();
    println!("asm={} threads={threads} frames={frames} seconds={seconds:.6} fps={:.2}",cfg!(feature="asm"),frames as f64/seconds);
    Ok(())
}
