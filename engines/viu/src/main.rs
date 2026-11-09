// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--preview") {
        furniture::preview::run(&args[1..])
    } else if args.first().is_some_and(|a| a == "--host") {
        let service = std::env::var("MIX_VIU_SERVICE").unwrap_or_else(|_| "shell".into());
        let comp = std::env::var("MIX_VIU_COMP_SERVICE").unwrap_or_else(|_| "comp".into());
        if let Err(error) = furniture::runtime::run(service, comp) {
            eprintln!("viu: {error}");
            std::process::exit(1);
        }
        Ok(())
    } else {
        viu::app_host::run(args)
    }
}
