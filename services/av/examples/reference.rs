//! Worker-only lossless reference export; shares the executor's pixel path.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("usage: reference TIMELINE.json")?;
    av::render::reference(std::path::Path::new(&path), std::io::stdout().lock())
}
