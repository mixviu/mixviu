// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional runtime faces shared by toolkit and scene hosts. No system discovery.
use egui::{FontData, FontDefinitions, FontFamily};
use std::path::{Path, PathBuf};

pub const FILES: &[&str] = &[
    "LiberationSans-Regular.ttf",
    "LiberationSans-Bold.ttf",
    "NotoSansMono-Regular.ttf",
    "NotoSans-Regular.ttf",
    "NotoSansMono-Medium.ttf",
    "NotoSansMono-Bold.ttf",
];

/// The single optional-family catalogue: family, CSS weight, bundled file.
pub const FACES: &[(&str, u16, &str)] = &[
    ("Liberation Sans", 400, FILES[0]),
    ("Liberation Sans", 700, FILES[1]),
    ("Noto Sans Mono", 400, FILES[2]),
    ("Noto Sans Mono", 500, FILES[4]),
    ("Noto Sans Mono", 700, FILES[5]),
    ("Noto Sans", 400, FILES[3]),
];

pub fn directory() -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../share/fonts");
    if cfg!(debug_assertions) && source.is_dir() {
        source
    } else {
        paths::share("fonts")
    }
}

/// Load the complete set atomically. Missing files never create substitute families.
pub fn register(fonts: &mut FontDefinitions, dir: &Path) -> std::io::Result<()> {
    let faces = FILES
        .iter()
        .map(|file| std::fs::read(dir.join(file)))
        .collect::<std::io::Result<Vec<_>>>()?;
    for (file, data) in FILES.iter().zip(faces) {
        fonts
            .font_data
            .insert((*file).into(), FontData::from_owned(data).into());
        fonts
            .families
            .insert(FontFamily::Name((*file).into()), vec![(*file).into()]);
    }
    for &(family, weight, primary) in FACES {
        let mut stack = vec![primary.into()];
        if primary != FILES[3] {
            stack.push(FILES[3].into());
        }
        fonts.families.insert(
            FontFamily::Name(format!("{family}/{weight}").into()),
            stack.clone(),
        );
        if weight == 400 {
            fonts
                .families
                .insert(FontFamily::Name(family.into()), stack);
        }
    }
    Ok(())
}

/// CSS Fonts 4 §5.2 static weight search, including fractional requests.
/// https://www.w3.org/TR/css-fonts-4/#font-style-matching
pub fn match_weight(request: f32, weights: impl Iterator<Item = u16>) -> Option<u16> {
    let priority = |weight: u16| {
        let w = f32::from(weight);
        if (400.0..=500.0).contains(&request) {
            if w >= request && w <= 500.0 {
                (0, w)
            } else if w < request {
                (1, -w)
            } else {
                (2, w)
            }
        } else if request < 400.0 {
            if w <= request { (0, -w) } else { (1, w) }
        } else if w >= request {
            (0, w)
        } else {
            (1, -w)
        }
    };
    weights.min_by(|a, b| {
        let a = priority(*a);
        let b = priority(*b);
        a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1))
    })
}

pub fn family(name: &str, request: f32) -> FontFamily {
    let weight = match_weight(request, FACES.iter().filter(|f| f.0 == name).map(|f| f.1));
    FontFamily::Name(
        weight
            .map_or_else(|| name.to_owned(), |w| format!("{name}/{w}"))
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn css_weight_search_matches_measured_faces_and_sparse_families() {
        for (name, expected) in [
            ("Liberation Sans", [400, 400, 700, 700]),
            ("Noto Sans Mono", [400, 500, 700, 700]),
        ] {
            for (request, expected) in [400., 500., 600., 700.].into_iter().zip(expected) {
                assert_eq!(
                    family(name, request),
                    FontFamily::Name(format!("{name}/{expected}").into())
                );
            }
        }
        for (request, expected) in [
            (200., 100),
            (350., 300),
            (450., 500),
            (500.1, 700),
            (800., 900),
            (1000., 900),
        ] {
            assert_eq!(
                match_weight(request, [700, 100, 900, 400, 300, 500].into_iter()),
                Some(expected)
            );
        }
        assert_eq!(match_weight(500., [700, 400].into_iter()), Some(400));
        assert_eq!(match_weight(400., [700, 500].into_iter()), Some(500));
        assert_eq!(match_weight(400., [].into_iter()), None);
    }
}
