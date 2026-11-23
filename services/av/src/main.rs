use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    match args.next().as_deref() {
        Some(command) if command == "audio" => match media_audio::cli::run(args) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                let _ = media_audio::cli::diagnostic(format_args!("{e}"));
                ExitCode::from(2)
            }
        },
        _ => {
            let _ = media_audio::cli::diagnostic(format_args!("usage: av audio TIMELINE.json --out FILE.webm [--wav FILE.wav]"));
            ExitCode::from(2)
        }
    }
}
