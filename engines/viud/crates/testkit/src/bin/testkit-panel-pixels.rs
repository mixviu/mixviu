// SPDX-License-Identifier: MIT OR Apache-2.0
//! P3 lab acceptance: count an output-specific model colour in the panel band.
use std::error::Error;

fn run() -> Result<bool, Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 || args.len() > 4 {
        return Err("usage: testkit-panel-pixels PNG RRGGBB [MIN_PIXELS] [BAND_HEIGHT]".into());
    }
    let hex = &args[1];
    if hex.len() != 6 || !hex.is_ascii() {
        return Err("expected RRGGBB".into());
    }
    let colour = [
        u8::from_str_radix(&hex[0..2], 16)?,
        u8::from_str_radix(&hex[2..4], 16)?,
        u8::from_str_radix(&hex[4..6], 16)?,
    ];
    let minimum: usize = args
        .get(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(10_000);
    let band: u32 = args.get(3).map(|v| v.parse()).transpose()?.unwrap_or(52);
    let image = image::ImageReader::open(&args[0])?.decode()?.to_rgba8();
    if band == 0 || band > image.height() {
        return Err("invalid band height".into());
    }
    let count = image
        .enumerate_pixels()
        .filter(|(_, y, p)| {
            *y >= image.height() - band && (0..3).all(|i| p[i].abs_diff(colour[i]) <= 2)
        })
        .count();
    let pass = count >= minimum;
    println!(
        "{{\"width\":{},\"height\":{},\"matching_pixels\":{count},\"minimum\":{minimum},\"pass\":{pass}}}",
        image.width(),
        image.height()
    );
    Ok(pass)
}

fn main() {
    match run() {
        Ok(true) => (),
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("testkit-panel-pixels: {error}");
            std::process::exit(2);
        }
    }
}
