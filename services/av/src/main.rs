use av::render;
use std::process::ExitCode;
mod decode;

const USAGE: &str = "usage: av render TIMELINE.json --out FILE.webm [--cpu N] [--mem BYTES] [--progress]\n       av audio TIMELINE.json --out FILE.webm [--wav FILE.wav] [--cpu N] [--progress]\n       av decode FILE.webm --y4m OUT.y4m [--wav OUT.wav] [--max-output-bytes N]\n       av probe FILE.webm";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    match args.next().as_deref() {
        Some(command) if command == "render" => match render::run(args) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                let _ = media_audio::cli::diagnostic(format_args!("{e}"));
                ExitCode::from(2)
            }
        },
        Some(command) if command == "audio" => match media_audio::cli::run(args) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                let _ = media_audio::cli::diagnostic(format_args!("{e}"));
                ExitCode::from(2)
            }
        },
        Some(command) if command == "decode" || command == "probe" => {
            match decode::run(command == "probe", args) {
                Ok(code) => ExitCode::from(code),
                Err(e) => {
                    let _ = media_audio::cli::diagnostic(format_args!("{e}"));
                    ExitCode::from(2)
                }
            }
        }
        _ => {
            let _ = media_audio::cli::diagnostic(format_args!("{USAGE}"));
            ExitCode::from(2)
        }
    }
}
