// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Prepare small scene icons before drawing. Rasters are decoded here, before
//! the pass that draws them, so the first frame already has pixels and
//! dimensions; SVGs keep their bytes and are rasterised at the size they are
//! drawn (`crate::view` uploads both as egui textures on its own context).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

/// A unique id per decoded asset, so a texture cache can tell a re-decoded
/// file from the same one.
fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Decoded RGBA pixels (unmultiplied), shared.
#[derive(Clone, Debug)]
pub(crate) struct Raster {
    pub id: u64,
    pub size: [usize; 2],
    pub rgba: Arc<[u8]>,
}

impl Raster {
    pub fn from_rgba(width: usize, height: usize, rgba: Vec<u8>) -> Self {
        Self {
            id: next_id(),
            size: [width, height],
            rgba: rgba.into(),
        }
    }
}

/// An SVG document, rasterised at draw size. `symbolic` icons take the
/// scene ink; full-colour artwork keeps its colours.
#[derive(Clone, Debug)]
pub(crate) struct Svg {
    pub id: u64,
    pub bytes: Arc<[u8]>,
    pub symbolic: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum Asset {
    Svg(Svg),
    Raster(Raster),
}

/// Rasterise `svg` to `width` x `height` pixels (premultiplied in egui's
/// form). A symbolic icon comes out white with its alpha, to be tinted.
pub(crate) fn rasterise(svg: &Svg, width: u32, height: u32) -> Option<egui::ColorImage> {
    let tree = resvg::usvg::Tree::from_data(&svg.bytes, &resvg::usvg::Options::default()).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width.max(1), height.max(1))?;
    let size = tree.size();
    let transform = resvg::tiny_skia::Transform::from_scale(
        width.max(1) as f32 / size.width(),
        height.max(1) as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let mut rgba = pixmap.take();
    if svg.symbolic {
        for pixel in rgba.chunks_exact_mut(4) {
            // Premultiplied white at the glyph's coverage.
            let a = pixel[3];
            pixel[0] = a;
            pixel[1] = a;
            pixel[2] = a;
        }
    }
    Some(egui::ColorImage::from_rgba_premultiplied(
        [width.max(1) as usize, height.max(1) as usize],
        &rgba,
    ))
}

pub(crate) type Prepared = BTreeMap<(String, String, String), Asset>;
type FileStamp = (Option<SystemTime>, u64);
type AssetCache =
    crate::cache::Cache<(String, u32, Option<String>), (Asset, Option<(PathBuf, FileStamp)>)>;

fn file_stamp(path: &Path) -> Option<FileStamp> {
    // Accepted limitation: coarse mtimes can hide same-size in-place writes.
    let meta = path.metadata().ok()?;
    Some((meta.modified().ok(), meta.len()))
}

pub(crate) fn key(id: &str, row: Option<&crate::view::Route>) -> (String, String, String) {
    (
        row.map_or("", |r| r.node.as_str()).into(),
        row.and_then(|r| r.item.as_ref())
            .and_then(|item| item["id"].as_str())
            .unwrap_or_default()
            .into(),
        id.into(),
    )
}

// Legacy visible panel images with an empty source use the shared catalogue.
// Hidden images remain hidden, including in older scene documents.
pub(crate) fn panel_fallback(id: &str) -> Option<&'static str> {
    match id {
        "st_notes_i" => Some("icon:bell"),
        "st_vol_i" => Some("icon:volume-2"),
        "st_net_i" => Some("icon:ethernet-port"),
        "launcher_icon" => Some("brand:cyclops"),
        _ => None,
    }
}

pub(crate) fn fallback_image(id: &str) -> Option<&'static str> {
    match id {
        "st_notes_f" => Some("st_notes_i"),
        "st_vol_f" => Some("st_vol_i"),
        "st_net_f" => Some("st_net_i"),
        "launcher_text" => Some("launcher_icon"),
        _ => None,
    }
}

/// Legacy panel fallbacks are image references as well as child edges.
/// Return only references not already placed by the ordinary children path.
pub(crate) fn fallback_reference(tree: &scene::ResolvedScene, id: &str) -> Option<&'static str> {
    if tree.name != "panel" {
        return None;
    }
    let target = fallback_image(id)?;
    let image = tree.nodes.get(target)?;
    if image.family != "image" || crate::view::flag(image, "hidden") {
        return None;
    }
    let mut pending = vec!["root"];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = tree.nodes.get(id) else {
            continue;
        };
        if crate::view::flag(node, "hidden") {
            continue;
        }
        if id == target {
            return None;
        }
        pending.extend(crate::templates::children(node));
    }
    Some(target)
}

pub(crate) fn prepare(
    tree: &scene::ResolvedScene,
    lists: &crate::templates::PreparedLists,
) -> Prepared {
    // Embedded catalogue sources do not depend on XDG or the filesystem.
    // Keep the shell's fixed glyphs off the search-path validation path too.
    let templates = crate::templates::template_ids(tree);
    let embedded = |id: &str, node: &scene::Node, row: bool| {
        if node.family != "image" || crate::view::flag(node, "hidden") {
            return true;
        }
        let source = crate::freedesktop::normalise(crate::templates::text(node, "src"));
        source.starts_with("icon:")
            || source == "brand:cyclops"
            || (!row
                && source.is_empty()
                && (tree.name != "panel"
                    || panel_fallback(id).is_none_or(|source| {
                        source.starts_with("icon:") || source == "brand:cyclops"
                    })))
    };
    if tree
        .nodes
        .iter()
        .all(|(id, node)| templates.contains(id) || embedded(id, node, false))
        && lists.values().all(|data| {
            data.instances
                .values()
                .all(|nodes| nodes.iter().all(|(id, node)| embedded(id, node, true)))
        })
    {
        static EMBEDDED: OnceLock<Mutex<AssetCache>> = OnceLock::new();
        return prepare_with(
            tree,
            lists,
            EMBEDDED.get_or_init(Mutex::default),
            |_, _, _| None,
        );
    }
    // Only the fixed production resolver shares this cache. Injected resolvers
    // must use their own cache so they cannot consume or replace its assets.
    static CACHE: OnceLock<Mutex<(u64, AssetCache)>> = OnceLock::new();
    crate::icons::with_resolver(|resolver, generation| {
        let mut cache = CACHE.get_or_init(Mutex::default).lock().unwrap();
        if cache.0 != generation {
            cache.0 = generation;
            cache.1.clear();
        }
        let assets = Mutex::new(std::mem::take(&mut cache.1));
        let prepared = prepare_with(tree, lists, &assets, |source, size, app| {
            if source.starts_with("mix:") {
                return paths::resolve(source).ok().filter(|path| path.is_file());
            }
            resolver.resolve_in_generation(source, size, 1, app)
        });
        cache.1 = assets.into_inner().unwrap();
        prepared
    })
}

fn prepare_with(
    tree: &scene::ResolvedScene,
    lists: &crate::templates::PreparedLists,
    cache: &Mutex<AssetCache>,
    resolve: impl Fn(&str, u32, Option<&str>) -> Option<PathBuf>,
) -> Prepared {
    let mut prepared = Prepared::new();
    let mut insert = |key, id: &str, node: &scene::Node, app: Option<&str>| {
        if node.family != "image"
            || node
                .ports
                .get("hidden")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
        {
            return;
        }
        let source = crate::freedesktop::normalise(crate::templates::text(node, "src"));
        let source = if source.is_empty() && tree.name == "panel" {
            panel_fallback(id).unwrap_or(source)
        } else {
            source
        };
        // Launcher services can leave src empty or return an obsolete path.
        // Keep the desktop ID available so local theme lookup can recover it.
        if source.is_empty() && app.is_none() {
            return;
        }
        let size = ["w", "h"].map(|p| {
            node.ports
                .get(p)
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(16.0)
        });
        let size = size[0].max(size[1]).ceil() as u32;
        let cache_key = (source.to_owned(), size, app.map(str::to_owned));
        if let Some(asset) = cache.lock().ok().and_then(|mut cache| {
            cache
                .get(&cache_key)
                .filter(|(_, file)| {
                    file.as_ref()
                        .is_none_or(|(path, stamp)| file_stamp(path) == Some(*stamp))
                })
                .map(|(asset, _)| asset.clone())
        }) {
            prepared.insert(key, asset);
            return;
        }
        let embedded: Option<(Arc<[u8]>, bool)> = if source == "brand:cyclops" {
            Some((
                include_bytes!("../../../share/brand/logos/alien-cyclops.svg")
                    .as_slice()
                    .into(),
                false,
            ))
        } else {
            source
                .strip_prefix("icon:")
                .and_then(viu::icons::Icon::from_name)
                .map(|icon| (viu::icons::source(icon).into_bytes().into(), true))
        };
        if let Some((bytes, symbolic)) = embedded {
            let asset = Asset::Svg(Svg {
                id: next_id(),
                bytes,
                symbolic,
            });
            let mut cache = cache.lock().unwrap();
            cache.insert(cache_key, (asset.clone(), None));
            prepared.insert(key, asset);
        } else if let Some(path) = resolve(source, size, app) {
            let stamp = file_stamp(&path);
            let asset = if crate::icons::is_svg(&path) {
                // Keep bytes and identity alive through layout/page switches.
                std::fs::read(&path).ok().map(|bytes| {
                    Asset::Svg(Svg {
                        id: next_id(),
                        bytes: bytes.into(),
                        symbolic: theme_glyph(&path),
                    })
                })
            } else {
                raster(&path).map(Asset::Raster)
            };
            if let Some(asset) = asset {
                if let Ok(mut cache) = cache.lock() {
                    cache.insert(
                        cache_key,
                        (asset.clone(), stamp.map(|stamp| (path.clone(), stamp))),
                    );
                }
                prepared.insert(key, asset);
            }
        }
    };
    fn visible<'a>(
        get: &impl Fn(&str) -> Option<&'a scene::Node>,
        id: &str,
        out: &mut BTreeSet<String>,
        tree: Option<&scene::ResolvedScene>,
    ) {
        let Some(node) = get(id) else {
            return;
        };
        if node
            .ports
            .get("hidden")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
            || !out.insert(id.into())
        {
            return;
        }
        for child in crate::templates::children(node) {
            visible(get, child, out, tree);
        }
        if let Some(target) = tree.and_then(|tree| fallback_reference(tree, id)) {
            visible(get, target, out, tree);
        }
    }
    let mut drawn = BTreeSet::new();
    visible(&|id| tree.nodes.get(id), "root", &mut drawn, Some(tree));
    let templates = crate::templates::template_ids(tree);
    for (id, node) in &tree.nodes {
        if drawn.contains(id) && !templates.contains(id) {
            insert(key(id, None), id, node, None);
        }
    }
    for (list, data) in lists {
        if !drawn.contains(list) {
            continue;
        }
        for (item, nodes) in &data.instances {
            let mut drawn = BTreeSet::new();
            visible(
                &|id| nodes.get(id),
                crate::templates::text(&tree.nodes[list], "row"),
                &mut drawn,
                None,
            );
            let app = tree
                .nodes
                .get(list)
                .and_then(|node| {
                    crate::templates::rows(node)
                        .iter()
                        .find(|row| row["id"].as_str() == Some(item.as_str()))
                })
                .and_then(|row| row["app_id"].as_str().or_else(|| row["id"].as_str()));
            for (id, node) in nodes {
                if drawn.contains(id) {
                    insert((list.clone(), item.clone(), id.clone()), id, node, app);
                }
            }
        }
    }
    prepared
}

type RasterCache = BTreeMap<PathBuf, (Option<SystemTime>, u64, Raster)>;

pub(crate) fn raster(path: &Path) -> Option<Raster> {
    static CACHE: OnceLock<Mutex<RasterCache>> = OnceLock::new();
    raster_with_cache(path, CACHE.get_or_init(Mutex::default))
}

fn raster_with_cache(path: &Path, cache: &Mutex<RasterCache>) -> Option<Raster> {
    let metadata = path.metadata().ok()?;
    let stamp = (metadata.modified().ok(), metadata.len());
    // Keep lookup, decoding and insertion atomic: concurrent misses must not
    // create different IDs for the same unchanged file.
    let mut cache = cache.lock().ok()?;
    if let Some((_, _, handle)) = cache
        .get(path)
        .filter(|(modified, len, _)| (*modified, *len) == stamp)
    {
        return Some(handle.clone());
    }
    let pixels = image::open(path).ok()?.into_rgba8();
    let handle = Raster::from_rgba(
        pixels.width() as usize,
        pixels.height() as usize,
        pixels.into_raw(),
    );
    if cache.len() >= 512 {
        // A live handle shares its pixel buffer with the cache. Only discard
        // unused entries, allowing the limit to grow while scenes hold icons.
        cache.retain(|_, (_, _, handle)| Arc::strong_count(&handle.rgba) > 1);
    }
    cache.insert(path.into(), (stamp.0, stamp.1, handle.clone()));
    Some(handle)
}

/// Symbolic icons and KDE's monochrome theme glyphs need the scene ink. Apps
/// retain their artwork; a light-theme network glyph must not disappear on
/// the dark launcher just because apps.icon supplied its absolute path.
pub(crate) fn theme_glyph(path: &Path) -> bool {
    if path
        .file_stem()
        .is_some_and(|name| name.to_string_lossy().ends_with("-symbolic"))
    {
        return true;
    }
    if path.components().any(|part| part.as_os_str() == "apps") {
        return false;
    }
    std::fs::read_to_string(path).is_ok_and(|source| source.contains("ColorScheme-Text"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_catalogue_names_all_exist() {
        let mut names = BTreeSet::new();
        for source in [
            include_str!("../../../share/scenes/panel/behaviour.mix"),
            include_str!("../../../share/scenes/panel/scene.mix"),
        ] {
            for tail in source.split("\"icon:").skip(1) {
                let name = tail.split('"').next().unwrap();
                assert!(
                    viu::icons::Icon::from_name(name).is_some(),
                    "unknown panel catalogue icon: {name}"
                );
                names.insert(name);
            }
        }
        assert!(names.len() >= 10, "the check must cover the panel mapping");
    }

    #[test]
    fn hidden_subtrees_and_rows_never_resolve_and_visible_assets_reuse_handles() {
        let tree = scene::resolve(
            &scene::parse(
                r#"---
scene: 1
name: hidden
citizen: test
---
```mix
root: {widget: "column", children: ["hidden", "rows", "shown"]}
hidden: {widget: "column", hidden: true, children: ["bad"]}
bad: {widget: "image", src: "hidden-parent"}
rows: {widget: "list", row: "row", row_height: 20, rows: [{id: "one", cells: []}]}
row: {widget: "column", hidden: true, children: ["row_icon"]}
row_icon: {widget: "image", src: "hidden-row"}
shown: {widget: "image", src: "visible", w: 20, h: 20}
```
"#,
            )
            .unwrap(),
        )
        .unwrap();
        let lists = crate::templates::validate_templates(&tree).unwrap();
        let cache = Mutex::new(AssetCache::new());
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedded-icon.svg");
        let first = prepare_with(&tree, &lists, &cache, |source, _, _| {
            assert_eq!(source, "visible");
            Some(path.clone())
        });
        assert_eq!(first.len(), 1);
        let second = prepare_with(&tree, &lists, &cache, |_, _, _| {
            panic!("unchanged inputs must reuse the asset")
        });
        let Some(Asset::Svg(a)) = first.get(&key("shown", None)) else {
            panic!("SVG");
        };
        let Some(Asset::Svg(b)) = second.get(&key("shown", None)) else {
            panic!("SVG");
        };
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn empty_launcher_sources_reach_desktop_lookup_and_prepare_real_pixels() {
        let document = scene::parse(
            r#"---
scene: 1
name: launcher
citizen: test
---
```mix
root: {widget: "column", children: ["apps"]}
apps: {widget: "list", row: "app", row_height: 31, rows: [
  {id: "dev.mixos.media.desktop", cells: []},
  {id: "bssh.desktop", cells: []},
  {id: "bvnc.desktop", cells: []},
  {id: "avahi-discover.desktop", cells: []}
]}
app: {widget: "row", children: ["icon"]}
icon: {widget: "image", src: "", w: 31, h: 31}
```
"#,
        )
        .expect("empty-source launcher scene must parse");
        let tree = scene::resolve(&document).expect("empty-source launcher scene must resolve");
        let lists = crate::templates::validate_templates(&tree)
            .expect("empty-source launcher row templates must validate");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/icon.png");
        let seen = std::cell::RefCell::new(Vec::new());
        let cache = Mutex::new(AssetCache::new());
        let images = prepare_with(&tree, &lists, &cache, |source, _, app| {
            assert!(source.is_empty());
            seen.borrow_mut().push(
                app.expect("launcher image must retain its desktop ID")
                    .to_owned(),
            );
            Some(path.clone())
        });
        assert_eq!(
            seen.borrow().len(),
            4,
            "empty sources must not bypass recovery"
        );
        for id in [
            "dev.mixos.media.desktop",
            "bssh.desktop",
            "bvnc.desktop",
            "avahi-discover.desktop",
        ] {
            assert!(matches!(
                images.get(&("apps".into(), id.into(), "icon".into())),
                Some(Asset::Raster(_))
            ));
        }
    }

    #[test]
    fn empty_panel_launcher_uses_embedded_colour_brand() {
        let tree = scene::resolve(&scene::parse(
            "---\nscene: 1\nname: panel\ncitizen: test\n---\n```mix\nroot: {widget: \"row\", children: [\"launcher_icon\"]}\nlauncher_icon: {widget: \"image\", src: \"\", w: 28, h: 28}\n```\n",
        ).unwrap()).unwrap();
        let lists = crate::templates::validate_templates(&tree).unwrap();
        let images = prepare_with(&tree, &lists, &Mutex::new(AssetCache::new()), |_, _, _| {
            panic!("catalogue icons never search the filesystem")
        });
        assert!(
            matches!(
                images.get(&key("launcher_icon", None)),
                Some(Asset::Svg(Svg {
                    symbolic: false,
                    ..
                }))
            ),
            "the cyclops keeps its colours"
        );
        let Some(Asset::Svg(svg)) = images.get(&key("launcher_icon", None)) else {
            panic!("brand SVG");
        };
        let pixels = rasterise(svg, 64, 64).unwrap();
        assert!(pixels.pixels.iter().any(|p| p.r() > 100
            && p.r() > p.g().saturating_mul(2)
            && p.r() > p.b().saturating_mul(2)));
        assert_eq!(fallback_image("launcher_text"), Some("launcher_icon"));
    }

    #[test]
    fn injected_resolvers_use_independent_asset_caches() {
        let tree = scene::resolve(&scene::parse(
            "---\nscene: 1\nname: icons\ncitizen: test\n---\n```mix\nroot: {widget: \"image\", src: \"same-source\", w: 31, h: 31}\n```\n",
        ).unwrap()).unwrap();
        let lists = crate::templates::validate_templates(&tree).unwrap();
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let raster_cache = Mutex::new(AssetCache::new());
        let svg_cache = Mutex::new(AssetCache::new());
        let root = key("root", None);
        let rasters = prepare_with(&tree, &lists, &raster_cache, |_, _, _| {
            Some(fixtures.join("icon.png"))
        });
        let svgs = prepare_with(&tree, &lists, &svg_cache, |_, _, _| {
            Some(fixtures.join("embedded-icon.svg"))
        });
        assert!(matches!(rasters.get(&root), Some(Asset::Raster(_))));
        assert!(matches!(svgs.get(&root), Some(Asset::Svg(_))));
        // Reusing a cache bypasses resolution and keeps the same decoded handle.
        let cached = prepare_with(&tree, &lists, &raster_cache, |_, _, _| {
            panic!("cached asset must bypass lookup")
        });
        let (Some(Asset::Raster(first)), Some(Asset::Raster(second))) =
            (rasters.get(&root), cached.get(&root))
        else {
            panic!("expected cached raster handles")
        };
        assert_eq!(first.id, second.id);
    }

    #[test]
    fn rasters_are_ready_in_the_first_frame_and_reuse_their_handle() {
        // A real PNG decoder path, independent of GPU upload timing.
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/icon.png");
        let cache = Mutex::new(RasterCache::new());
        let first = raster_with_cache(&path, &cache).unwrap();
        let second = raster_with_cache(&path, &cache).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!((first.size, first.rgba.len()), ([1, 1], 4));
        assert!(first.rgba[3] > 0);
        assert!(raster_with_cache(Path::new("/missing/scene-icon.png"), &cache).is_none());
    }

    #[test]
    fn raster_cache_pressure_preserves_live_handles_and_prunes_unused_entries() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/icon.png");
        // A distinct cache key for the same fixture; no temporary files needed.
        let other = path.parent().unwrap().join("../fixtures/icon.png");
        for keep_live in [false, true] {
            let cache = Mutex::new(RasterCache::new());
            let first = raster_with_cache(&path, &cache).unwrap();
            let mut live = Vec::new();
            {
                let mut entries = cache.lock().unwrap();
                for index in 0..511 {
                    let handle = Raster::from_rgba(1, 1, vec![0; 4]);
                    if keep_live {
                        live.push(handle.clone());
                    }
                    entries.insert(
                        PathBuf::from(format!("unused-{index}.png")),
                        (None, 0, handle),
                    );
                }
                assert_eq!(entries.len(), 512);
            }
            let _other = raster_with_cache(&other, &cache).unwrap();
            let second = raster_with_cache(&path, &cache).unwrap();
            assert_eq!(first.id, second.id, "keep_live={keep_live}");
            assert_eq!(cache.lock().unwrap().len(), if keep_live { 513 } else { 2 });
            drop(live);
        }
    }

    #[test]
    fn concurrent_raster_lookups_share_one_handle() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/icon.png");
        let cache = Mutex::new(RasterCache::new());
        let start = std::sync::Barrier::new(8);
        let handles = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        start.wait();
                        raster_with_cache(&path, &cache).unwrap()
                    })
                })
                .collect();
            threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(handles.iter().all(|handle| handle.id == handles[0].id));
        assert_eq!(cache.lock().unwrap().len(), 1);
    }

    #[test]
    fn png_decode_produces_painted_rgba_pixels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/icon.png");
        let raster = raster(&path).expect("PNG must be decoded before drawing");
        assert_eq!((raster.size, raster.rgba.len()), ([1, 1], 4));
        assert!(
            raster
                .rgba
                .chunks_exact(4)
                .any(|pixel| pixel[0] > 0 && pixel[3] > 0),
            "blank decoded PNG"
        );
    }

    #[test]
    fn svg_embedded_png_rasterises_to_non_transparent_pixels() {
        // The draw-time rasteriser, raster images embedded in SVG included.
        let svg = Svg {
            id: next_id(),
            bytes: include_bytes!("../tests/fixtures/embedded-icon.svg")
                .as_slice()
                .into(),
            symbolic: false,
        };
        let image = rasterise(&svg, 8, 8).expect("rasterised");
        assert_eq!(image.size, [8, 8]);
        assert!(
            image
                .pixels
                .iter()
                .any(|pixel| pixel.r() > 0 && pixel.a() > 0),
            "blank embedded PNG"
        );
        // A symbolic icon comes out white at its coverage, ready to tint.
        let symbolic = rasterise(
            &Svg {
                symbolic: true,
                ..svg
            },
            8,
            8,
        )
        .expect("rasterised");
        assert!(
            symbolic
                .pixels
                .iter()
                .all(|pixel| pixel.r() == pixel.a() && pixel.g() == pixel.a())
        );
    }
}
