// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional runtime faces shared by toolkit and scene hosts. No system discovery.
use egui::{FontData, FontDefinitions, FontFamily};
use std::path::{Path, PathBuf};

pub const FILES: &[&str] = &[
    "LiberationSans-Regular.ttf", "LiberationSans-Bold.ttf",
    "NotoSansMono-Regular.ttf", "NotoSans-Regular.ttf",
];

pub fn directory() -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../share/fonts");
    if cfg!(debug_assertions) && source.is_dir() { source } else { paths::share("fonts") }
}

/// Load the complete set atomically. Missing files never create substitute families.
pub fn register(fonts: &mut FontDefinitions, dir: &Path) -> std::io::Result<()> {
    let faces = FILES.iter().map(|file| std::fs::read(dir.join(file)))
        .collect::<std::io::Result<Vec<_>>>()?;
    for (file, data) in FILES.iter().zip(faces) {
        fonts.font_data.insert((*file).into(), FontData::from_owned(data).into());
        fonts.families.insert(FontFamily::Name((*file).into()), vec![(*file).into()]);
    }
    for (family, primary) in [
        ("Liberation Sans", FILES[0]), ("Liberation Sans Bold", FILES[1]),
        ("Noto Sans Mono", FILES[2]), ("Noto Sans", FILES[3]),
    ] {
        let mut stack = vec![primary.into()];
        if primary != FILES[3] { stack.push(FILES[3].into()); }
        fonts.families.insert(FontFamily::Name(family.into()), stack);
    }
    Ok(())
}

/// Chrome's static weight mapping; no synthetic emboldening.
pub fn family(name: &str, weight: u16) -> FontFamily {
    FontFamily::Name(if name == "Liberation Sans" && weight >= 600 {
        "Liberation Sans Bold"
    } else { name }.into())
}
