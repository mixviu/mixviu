// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() {
    let service=std::env::var("VIU_SERVICE").unwrap_or_else(|_|"shell".into());
    let comp=std::env::var("VIU_COMP_SERVICE").unwrap_or_else(|_|"comp".into());
    if let Err(error)=viu::runtime::run(service,comp) {eprintln!("viu: {error}");std::process::exit(1);}
}
