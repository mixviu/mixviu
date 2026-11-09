// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! freedesktop icon-theme lookup (the Icon Theme and Desktop Entry
//! specifications), std only.
//!
//! A [`Lookup`] is a set of icon roots, pixmap directories, application
//! directories and a theme name; the caller supplies them
//! ([`Lookup::new`], [`Lookup::in_data_dirs`]) or takes the XDG
//! environment's ([`Lookup::from_xdg`]). [`Lookup::find`] resolves an icon
//! name at a size through the theme, its `Inherits` chain, the fallback
//! themes and the pixmap directories, with the `-symbolic` and plain
//! variants tried both ways. [`Lookup::resolve`] adds the desktop-entry
//! route (a `.desktop` file's `Icon=`, or its category) and the generic
//! placeholder icons. A [`Resolver`] caches resolutions.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

pub(crate) fn normalise(source: &str) -> &str {
    let source = source.trim();
    source.strip_prefix("file://").unwrap_or(source)
}

/// Themes searched after the selected one, in order.
pub const FALLBACK_THEMES: [&str; 3] = ["hicolor", "Adwaita", "AdwaitaLegacy"];

/// The theme used when neither the environment nor a desktop setting names
/// one.
pub const DEFAULT_THEME: &str = "Adwaita";

/// Icon names tried when nothing else resolves, in order.
pub const PLACEHOLDER_ICONS: [&str; 2] = ["application-x-executable", "application-default-icon"];

const IMAGE_EXTENSIONS: [&str; 8] = ["svg", "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico"];

/// Whether the path has an `.svg` extension (any case).
pub fn is_svg(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

/// Whether the path is an existing file with an image extension.
pub fn is_image_file(path: &Path) -> bool {
    path.is_file()
        && path.extension().is_some_and(|ext| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|kind| ext.eq_ignore_ascii_case(kind))
        })
}

type Ini = BTreeMap<String, BTreeMap<String, String>>;

/// A minimal INI reader for `index.theme`, `.desktop` and settings files:
/// `[section]` headers, `key=value` lines, `#`/`;` comments, quotes
/// stripped. Missing or unreadable files read as empty.
fn ini(path: &Path) -> Ini {
    let mut sections = Ini::new();
    let mut section = String::new();
    for line in std::fs::read_to_string(path).unwrap_or_default().lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            section = name.into();
        } else if let Some((key, value)) = line.split_once('=') {
            sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().into(), value.trim().trim_matches('"').into());
        }
    }
    sections
}

/// Where icons are looked up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lookup {
    /// Icon-theme roots (`.../icons`), searched in order.
    pub roots: Vec<PathBuf>,
    /// Legacy flat icon directories (`.../pixmaps`).
    pub pixmaps: Vec<PathBuf>,
    /// Desktop-entry directories (`.../applications`).
    pub applications: Vec<PathBuf>,
    /// The selected theme, searched before [`FALLBACK_THEMES`].
    pub theme: String,
}

impl Lookup {
    pub fn new(
        theme: impl Into<String>,
        roots: Vec<PathBuf>,
        pixmaps: Vec<PathBuf>,
        applications: Vec<PathBuf>,
    ) -> Self {
        Self {
            roots,
            pixmaps,
            applications,
            theme: theme.into(),
        }
    }

    /// The `icons`, `pixmaps` and `applications` subdirectories of each
    /// data directory, in order.
    pub fn in_data_dirs(
        theme: impl Into<String>,
        data_dirs: impl IntoIterator<Item = PathBuf>,
    ) -> Self {
        let mut lookup = Self::new(theme, Vec::new(), Vec::new(), Vec::new());
        for dir in data_dirs {
            lookup.roots.push(dir.join("icons"));
            lookup.pixmaps.push(dir.join("pixmaps"));
            lookup.applications.push(dir.join("applications"));
        }
        lookup
    }

    /// The XDG base directories (`~/.icons`, `XDG_DATA_HOME`,
    /// `XDG_DATA_DIRS`) and the theme named by `XDG_ICON_THEME`, else the
    /// GTK 4/3 `settings.ini`, else `kdeglobals`, else [`DEFAULT_THEME`].
    pub fn from_xdg() -> Self {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let dirs = std::env::var_os("XDG_DATA_DIRS")
            .filter(|dirs| !dirs.is_empty())
            .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
        let mut lookup = Self::in_data_dirs(
            String::new(),
            std::iter::once(data.clone()).chain(std::env::split_paths(&dirs)),
        );
        lookup.roots.insert(0, home.join(".icons"));
        let gtk = ["gtk-4.0/settings.ini", "gtk-3.0/settings.ini"]
            .into_iter()
            .find_map(|file| {
                ini(&config.join(file))
                    .get("Settings")
                    .and_then(|settings| settings.get("gtk-icon-theme-name"))
                    .cloned()
            });
        let kde = ini(&config.join("kdeglobals"))
            .get("Icons")
            .and_then(|icons| icons.get("Theme"))
            .cloned();
        lookup.theme = std::env::var("XDG_ICON_THEME")
            .ok()
            .or(gtk)
            .or(kde)
            .unwrap_or_else(|| DEFAULT_THEME.into());
        lookup
    }

    /// The `Icon=` of the desktop entry `id` (with or without `.desktop`),
    /// searched through the application directories (nested entries flatten
    /// `/` to `-` in their ID). An entry without `Icon=` but with an
    /// `AudioVideo` category gives `applications-multimedia`.
    pub fn application_icon(&self, id: &str) -> Option<String> {
        if id.is_empty() || id.contains(['/', '\\']) || id == "." || id == ".." {
            return None;
        }
        let file = if id.ends_with(".desktop") {
            id.to_owned()
        } else {
            format!("{id}.desktop")
        };
        self.applications.iter().find_map(|dir| {
            let direct = dir.join(&file);
            let path = if direct.is_file() {
                Some(direct)
            } else {
                desktop_file(dir, dir, &file, 0)
            }?;
            let entry = ini(&path);
            let fields = entry.get("Desktop Entry")?;
            fields
                .get("Icon")
                .filter(|icon| !icon.is_empty())
                .cloned()
                .or_else(|| {
                    fields.get("Categories").and_then(|categories| {
                        list(categories).find_map(|category| match category {
                            "AudioVideo" => Some("applications-multimedia".into()),
                            _ => None,
                        })
                    })
                })
        })
    }

    /// The file of icon `name` nearest `size`, through the theme chain, the
    /// pixmaps and the `-symbolic`/plain alternates. An absolute image path
    /// (optionally `file://`) is returned as is; a name with a path
    /// separator never leaves the icon directories.
    pub fn find(&self, name: &str, size: u32) -> Option<PathBuf> {
        let name = normalise(name);
        let path = Path::new(name);
        if path.is_absolute() && is_image_file(path) {
            return Some(path.into());
        }
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return None;
        }
        if let Some(path) = self.themed(name, size) {
            return Some(path);
        }
        for dir in &self.pixmaps {
            if let Some(path) = icon_in(dir, name) {
                return Some(path);
            }
        }
        // Some themes install shell glyphs only as `*-symbolic`; others only
        // plain. Try the other spelling once, never bouncing back.
        let stem = name
            .strip_suffix(".svg")
            .or_else(|| name.strip_suffix(".png"))
            .unwrap_or(name);
        if let Some(plain) = stem.strip_suffix("-symbolic") {
            if let Some(path) = self.themed(plain, size) {
                return Some(path);
            }
        } else if let Some(path) = self.find(&format!("{stem}-symbolic"), size) {
            return Some(path);
        }
        // Equivalent shell glyph names across the common themes.
        let alias = match stem.strip_suffix("-symbolic").unwrap_or(stem) {
            "notifications" | "notification-active" | "notification-inactive" => {
                Some("preferences-system-notifications-symbolic")
            }
            "network-wired-activated" => Some("network-wired-symbolic"),
            "desktop" | "user-desktop" | "display" => Some("video-display-symbolic"),
            _ => None,
        };
        alias.and_then(|alias| self.find(alias, size))
    }

    /// The selected theme, then the fallback themes, each with its
    /// `Inherits` chain.
    fn themed(&self, name: &str, size: u32) -> Option<PathBuf> {
        let mut seen = BTreeSet::new();
        std::iter::once(self.theme.as_str())
            .chain(FALLBACK_THEMES)
            .find_map(|theme| self.theme_icon(theme, name, size, &mut seen))
    }

    fn theme_icon(
        &self,
        theme: &str,
        name: &str,
        size: u32,
        seen: &mut BTreeSet<String>,
    ) -> Option<PathBuf> {
        if theme.is_empty()
            || theme.contains(['/', '\\'])
            || !seen.insert(theme.into())
            || seen.len() > 32
        {
            return None;
        }
        let mut candidates = Vec::new();
        let mut inherited = Vec::new();
        for root in &self.roots {
            let base = root.join(theme);
            let index = ini(&base.join("index.theme"));
            if let Some(header) = index.get("Icon Theme") {
                if let Some(names) = header.get("Inherits") {
                    inherited.extend(list(names).map(str::to_owned));
                }
                for key in ["Directories", "ScaledDirectories"] {
                    for dir in header.get(key).into_iter().flat_map(|dirs| list(dirs)) {
                        // Only the directories the theme declares, with no
                        // traversal and no scan of thousands of icons.
                        if !Path::new(dir)
                            .components()
                            .all(|part| matches!(part, Component::Normal(_)))
                        {
                            continue;
                        }
                        let fields = index.get(dir);
                        let number = |key: &str, default: u32| {
                            fields
                                .and_then(|fields| fields.get(key))
                                .and_then(|value| value.parse::<u32>().ok())
                                .unwrap_or(default)
                        };
                        let nominal = number("Size", size);
                        let scale = number("Scale", 1).max(1);
                        let kind = fields
                            .and_then(|fields| fields.get("Type"))
                            .map(String::as_str)
                            .unwrap_or("Threshold");
                        let (min, max) = match kind {
                            "Scalable" => (number("MinSize", nominal), number("MaxSize", nominal)),
                            "Fixed" => (nominal, nominal),
                            _ => (
                                nominal.saturating_sub(number("Threshold", 2)),
                                nominal.saturating_add(number("Threshold", 2)),
                            ),
                        };
                        let distance = min
                            .saturating_mul(scale)
                            .saturating_sub(size)
                            .max(size.saturating_sub(max.saturating_mul(scale)));
                        candidates.push((distance, base.join(dir)));
                    }
                }
            }
            // Packages and user overlays install directories the index
            // omits; unindexed themes have no index at all. Read directory
            // names (`48x48`, `scalable`, `symbolic`), not icon files.
            let mut installed = Vec::new();
            for entry in std::fs::read_dir(&base).into_iter().flatten().flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                let distance = if matches!(name.as_ref(), "scalable" | "symbolic") {
                    0
                } else {
                    let (dimensions, scale) = name.split_once('@').unwrap_or((&name, "1"));
                    let Some((w, h)) = dimensions.split_once('x') else {
                        continue;
                    };
                    let (Ok(w), Ok(h), Ok(scale)) =
                        (w.parse::<u32>(), h.parse::<u32>(), scale.parse::<u32>())
                    else {
                        continue;
                    };
                    if w != h || scale == 0 {
                        continue;
                    }
                    w.saturating_mul(scale).abs_diff(size)
                };
                for context in std::fs::read_dir(entry.path())
                    .into_iter()
                    .flatten()
                    .flatten()
                {
                    if context.file_type().is_ok_and(|kind| kind.is_dir()) {
                        installed.push((distance, context.path()));
                    }
                }
            }
            installed.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            candidates.extend(installed);
            candidates.push((u32::MAX, base));
        }
        // A stable sort keeps root order among equal distances.
        candidates.sort_by_key(|(distance, _)| *distance);
        for (_, dir) in candidates {
            if let Some(path) = icon_in(&dir, name) {
                return Some(path);
            }
        }
        inherited
            .iter()
            .find_map(|theme| self.theme_icon(theme, name, size, seen))
    }

    /// The icon for `source` (a name or path), else for the desktop entry
    /// `app` (its ID as an icon name, then its `Icon=`), else the file name
    /// of a stale path, else a placeholder icon.
    pub fn resolve(&self, source: &str, size: u32, app: Option<&str>) -> Option<PathBuf> {
        let source = normalise(source);
        self.find(source, size)
            .or_else(|| app.and_then(|id| self.find(id.trim_end_matches(".desktop"), size)))
            .or_else(|| {
                app.and_then(|id| self.application_icon(id))
                    .and_then(|name| self.find(&name, size))
            })
            .or_else(|| {
                Path::new(source)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| self.find(name, size))
            })
            .or_else(|| {
                PLACEHOLDER_ICONS
                    .iter()
                    .find_map(|name| self.find(name, size))
            })
    }
}

/// A comma, semicolon or whitespace separated list.
fn list(value: &str) -> impl Iterator<Item = &str> {
    value
        .split([',', ';'])
        .flat_map(str::split_whitespace)
        .filter(|name| !name.is_empty())
}

/// A desktop entry whose flattened ID (subdirectories joined with `-`) is
/// `id`. Only real directories are entered, so a symlink can neither loop
/// nor escape the root.
fn desktop_file(root: &Path, dir: &Path, id: &str, depth: usize) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir).ok()?.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            if let Some(path) = desktop_file(root, &path, id, depth + 1) {
                return Some(path);
            }
        } else if path
            .strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .replace('/', "-")
            == id
        {
            return Some(path);
        }
    }
    None
}

/// `name` in `dir`, as given or with the `.png`/`.svg` extension swapped
/// (an `Icon=firefox.svg` whose installed file is `firefox.png`).
fn icon_in(dir: &Path, name: &str) -> Option<PathBuf> {
    let path = dir.join(name);
    if is_image_file(&path) {
        return Some(path);
    }
    let stem = Path::new(name)
        .extension()
        .filter(|ext| {
            ["svg", "png"]
                .iter()
                .any(|kind| ext.eq_ignore_ascii_case(kind))
        })
        .and_then(|_| Path::new(name).file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    ["png", "svg"]
        .into_iter()
        .map(|ext| dir.join(format!("{stem}.{ext}")))
        .find(|path| is_image_file(path))
}

type Key = (String, u32, u32, String, Option<String>);

// Reuse the decoded-file cache's metadata validation strategy. Directory
// mtimes detect additions/removals; index files detect inheritance edits.
// Retain missing roots too, so installing a previously absent theme works.
type Stamp = (SystemTime, u64, Option<(SystemTime, u64)>);
type Stamps = BTreeMap<PathBuf, Option<Stamp>>;
const INVENTORY_LIMIT: usize = 4096;
const INVENTORY_WORK_LIMIT: usize = 65536;

#[derive(Debug)]
struct Inventory {
    stamps: Stamps,
    complete: bool,
    checked: Instant,
}

fn stamp(path: &Path) -> Option<Stamp> {
    let meta = path.symlink_metadata().ok()?;
    let target = meta
        .file_type()
        .is_symlink()
        .then(|| {
            path.metadata()
                .ok()
                .and_then(|m| Some((m.modified().ok()?, m.len())))
        })
        .flatten();
    // (mtime, len) can miss same-size in-place writes on coarse-mtime filesystems.
    Some((meta.modified().ok()?, meta.len(), target))
}

fn inventory(
    path: &Path,
    root: &Path,
    depth: usize,
    stamps: &mut Stamps,
    visited: &mut BTreeSet<PathBuf>,
    work: &mut usize,
) -> bool {
    if stamps.len() >= INVENTORY_LIMIT || *work >= INVENTORY_WORK_LIMIT {
        return false;
    }
    // Charge each monitored path once, never the untracked icon files or
    // both a directory's parent enumeration and its recursive visit.
    if !stamps.contains_key(path) { *work += 1; }
    stamps.insert(path.into(), stamp(path));
    let meta = match path.symlink_metadata() {
        Ok(meta) => meta,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    };
    let canonical = match path.canonicalize() {
        Ok(path) => path,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    };
    if !canonical.starts_with(root) {
        // External directory aliases cannot be monitored completely without
        // escaping the configured roots. Never enter them; use timed freshness.
        return !path.is_dir();
    }
    if !(meta.is_dir() || (meta.file_type().is_symlink() && path.is_dir())) {
        return true;
    }
    if depth > 16 {
        return false;
    }
    if !visited.insert(canonical) {
        return true;
    }
    // There is a one-level canonicalise/read_dir TOCTOU here. This is a
    // bounded, read-only inventory of same-user search paths, not a sandbox.
    let mut complete = true;
    let Ok(entries) = std::fs::read_dir(path) else { return false; };
    for entry in entries {
        let Ok(entry) = entry else { complete = false; continue; };
        let Ok(kind) = entry.file_type() else { complete = false; continue; };
        if kind.is_dir() || kind.is_symlink() {
            complete &= inventory(&entry.path(), root, depth + 1, stamps, visited, work);
        } else if entry.file_name() == "index.theme"
            || entry.path().extension().is_some_and(|ext| ext == "desktop")
        {
            if stamps.len() >= INVENTORY_LIMIT || *work >= INVENTORY_WORK_LIMIT {
                return false;
            }
            if !stamps.contains_key(&entry.path()) { *work += 1; }
            stamps.insert(entry.path(), stamp(&entry.path()));
        }
    }
    complete
}

fn snapshot(lookup: &Lookup) -> Inventory {
    let mut stamps = Stamps::new();
    let mut visited = BTreeSet::new();
    let mut work = 0;
    let mut complete = true;
    for path in lookup
        .roots
        .iter()
        .chain(&lookup.pixmaps)
        .chain(&lookup.applications)
    {
        let root = path.canonicalize().unwrap_or_else(|_| path.clone());
        complete &= inventory(path, &root, 0, &mut stamps, &mut visited, &mut work);
    }
    Inventory {
        stamps,
        complete,
        checked: Instant::now(),
    }
}

/// A [`Lookup`] with a bounded cache of resolutions, re-checked against
/// the file system on every hit.
#[derive(Debug)]
pub struct Resolver {
    lookup: Lookup,
    cache: Mutex<crate::cache::Cache<Key, Option<PathBuf>>>,
    stamps: Mutex<Inventory>,
    generation: std::sync::atomic::AtomicU64,
    single_check: Mutex<Instant>,
}

impl Resolver {
    pub fn new(lookup: Lookup) -> Self {
        let stamps = snapshot(&lookup);
        Self {
            lookup,
            cache: Mutex::new(crate::cache::Cache::new()),
            stamps: Mutex::new(stamps),
            generation: std::sync::atomic::AtomicU64::new(1),
            single_check: Mutex::new(Instant::now()),
        }
    }

    #[cfg(test)]
    pub fn lookup(&self) -> &Lookup {
        &self.lookup
    }

    /// Validate search paths once per preparation batch, before asset reuse.
    /// Complete inventories validate each batch. Over-limit/partial inventories
    /// retain caches for up to one second, then resnapshot and invalidate once.
    /// No thread or watcher service is created.
    pub fn generation(&self) -> u64 {
        let mut stamps = self.stamps.lock().unwrap();
        let refresh = if stamps.complete {
            stamps.stamps.iter().any(|(path, old)| stamp(path) != *old)
        } else {
            stamps.checked.elapsed() >= Duration::from_secs(1)
        };
        if refresh {
            self.cache.lock().unwrap().clear();
            *stamps = snapshot(&self.lookup);
            self.generation
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// [`Lookup::resolve`], cached by name, size, scale, theme and app.
    /// Single callers share freshness sweeps (at most every 250 ms).
    /// Batch callers use generation() for immediate explicit validation.
    pub fn resolve(&self, source: &str, size: u32, app: Option<&str>) -> Option<PathBuf> {
        let due = {
            let mut last = self.single_check.lock().unwrap();
            let due = last.elapsed() >= Duration::from_millis(250);
            if due {
                *last = Instant::now();
            }
            due
        };
        if due {
            self.generation();
        }
        self.resolve_in_generation(source, size, 1, app)
    }

    /// Batch callers validate generation once, then use this for each image.
    pub fn resolve_in_generation(
        &self,
        source: &str,
        size: u32,
        scale: u32,
        app: Option<&str>,
    ) -> Option<PathBuf> {
        let source = normalise(source);
        let path = Path::new(source);
        if path.is_absolute() {
            if is_image_file(path) { return Some(path.into()); }
            // Explicit paths may be outside every search root. Stat on every
            // call, and cache only the basename/app recovery, never this path.
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            return self.resolve_in_generation(name, size, scale, app);
        }
        let key = (
            source.to_owned(),
            size,
            scale,
            self.lookup.theme.clone(),
            app.map(str::to_owned),
        );
        if let Some(path) = self.cache.lock().ok().and_then(|mut cache| {
            cache
                .get(&key)
                .filter(|path| path.as_ref().is_none_or(|path| is_image_file(path)))
                .cloned()
        }) {
            return path;
        }
        let path = self.lookup.resolve(source, size.saturating_mul(scale), app);
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, path.clone());
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn inventory_never_enters_an_external_directory_symlink() {
        let scratch = Scratch::new();
        let outside = Scratch::new();
        for i in 0..5000 {
            std::fs::create_dir_all(outside.0.join(format!("large/{i}/nested"))).unwrap();
        }
        scratch.write("icons/Fixture/32x32/apps/icon.svg", "svg");
        let link = scratch.0.join("icons/external");
        std::os::unix::fs::symlink(&outside.0, &link).unwrap();
        let resolver = Resolver::new(scratch.lookup("Fixture"));
        let inventory = resolver.stamps.lock().unwrap();
        assert!(!inventory.complete, "external trees use timed freshness");
        assert!(inventory.stamps.len() < 20);
        assert!(
            inventory
                .stamps
                .keys()
                .all(|path| !path.starts_with(&link) || path == &link)
        );
        drop(inventory);
        resolver.resolve_in_generation("missing", 20, 1, None);
        assert_eq!(resolver.cache.lock().unwrap().len(), 1);
    }

    #[test]
    fn inventory_limit_retains_caches_until_the_one_second_refresh() {
        let scratch = Scratch::new();
        for i in 0..INVENTORY_LIMIT + 1 {
            std::fs::create_dir_all(scratch.0.join(format!("icons/{i}"))).unwrap();
        }
        let resolver = Resolver::new(scratch.lookup("Fixture"));
        let inventory = resolver.stamps.lock().unwrap();
        assert!(!inventory.complete);
        assert!(inventory.stamps.len() <= INVENTORY_LIMIT);
        drop(inventory);
        resolver.resolve_in_generation("later", 20, 1, None);
        assert_eq!(resolver.cache.lock().unwrap().len(), 1);
        let generation = resolver.generation();
        scratch.write("icons/Fixture/32x32/apps/later.svg", "svg");
        resolver.stamps.lock().unwrap().checked = Instant::now();
        assert_eq!(resolver.generation(), generation);
        assert!(resolver.resolve_in_generation("later", 20, 1, None).is_none());
        resolver.stamps.lock().unwrap().checked = Instant::now() - Duration::from_secs(2);
        assert!(resolver.generation() > generation);
        assert!(resolver.resolve_in_generation("later", 20, 1, None).is_some());
    }

    #[test]
    fn inventory_charges_only_monitored_paths_once() {
        let scratch = Scratch::new();
        for i in 0..1000 { scratch.write(&format!("icons/Fixture/32x32/apps/unused-{i}.svg"), "svg"); }
        let root = scratch.0.join("icons");
        let mut stamps = Stamps::new();
        let mut visited = BTreeSet::new();
        let mut work = 0;
        assert!(inventory(&root, &root.canonicalize().unwrap(), 0, &mut stamps, &mut visited, &mut work));
        assert_eq!(work, stamps.len());
        assert_eq!(work, 4, "icons/theme/size/context only; SVG files are not stamped");
    }

    #[test]
    fn single_calls_share_freshness_checks_and_batch_normalisation() {
        let scratch = Scratch::new();
        scratch.write("icons/Fixture/32x32/apps/named.svg", "svg");
        let resolver = Resolver::new(scratch.lookup("Fixture"));
        let path = scratch.0.join("icons/Fixture/32x32/apps/named.svg");
        assert_eq!(
            resolver.resolve_in_generation("  named  ", 32, 1, None),
            Some(path.clone())
        );
        assert_eq!(resolver.resolve("named", 32, None), Some(path.clone()));
        assert_eq!(resolver.cache.lock().unwrap().len(), 1);
        assert_eq!(
            resolver.resolve_in_generation(&format!("  file://{}  ", path.display()), 32, 1, None),
            Some(path)
        );
        assert!(resolver.resolve("later", 32, None).is_none());
        *resolver.single_check.lock().unwrap() = Instant::now();
        let generation = resolver
            .generation
            .load(std::sync::atomic::Ordering::Relaxed);
        scratch.write("icons/Fixture/32x32/apps/later.svg", "svg");
        assert!(
            resolver.resolve("later", 32, None).is_none(),
            "single-call sweeps are rate limited"
        );
        assert_eq!(
            resolver
                .generation
                .load(std::sync::atomic::Ordering::Relaxed),
            generation
        );
        assert!(
            resolver.generation() > generation,
            "explicit batches validate immediately"
        );
        assert!(resolver.resolve("later", 32, None).is_some());
    }

    #[cfg(unix)]
    #[test]
    fn a_broken_icon_symlink_is_invalidated_when_its_target_appears() {
        let scratch = Scratch::new();
        scratch.write("icons/Fixture/32x32/apps/existing.svg", "svg");
        let target = scratch.0.join("external.svg");
        let link = scratch.0.join("icons/Fixture/32x32/apps/linked.svg");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let resolver = Resolver::new(scratch.lookup("Fixture"));
        assert!(resolver.resolve("linked", 32, None).is_none());
        let generation = resolver.generation();
        std::fs::write(target, "svg").unwrap();
        assert!(resolver.generation() > generation);
        assert_eq!(resolver.resolve("linked", 32, None), Some(link));
    }

    #[test]
    fn misses_are_bounded_and_new_files_invalidate_them() {
        let scratch = Scratch::new();
        scratch.write("icons/Fixture/32x32/status/existing.svg", "svg");
        let resolver = Resolver::new(scratch.lookup("Fixture"));
        assert!(resolver.resolve("later", 20, None).is_none());
        let generation = resolver.generation();
        assert!(resolver.resolve("later", 20, None).is_none());
        assert_eq!(resolver.generation(), generation);
        assert!(resolver.cache.lock().unwrap().values().all(Option::is_none));
        scratch.write("icons/Fixture/32x32/status/later.svg", "svg");
        assert!(resolver.generation() > generation);
        assert_eq!(
            resolver.resolve("later", 20, None),
            Some(scratch.0.join("icons/Fixture/32x32/status/later.svg"))
        );
        for i in 0..crate::cache::CAPACITY + 5 {
            resolver.resolve_in_generation(&format!("missing-{i}"), 20, 1, None);
        }
        assert!(resolver.cache.lock().unwrap().len() <= crate::cache::CAPACITY);
        resolver.resolve_in_generation("missing", 20, 1, None);
        resolver.resolve_in_generation("missing", 20, 2, None);
        resolver.resolve_in_generation("missing", 24, 1, None);
        let cache = resolver.cache.lock().unwrap();
        assert!(cache.contains_key(&("missing".into(), 20, 2, "Fixture".into(), None)));
        assert!(cache.contains_key(&("missing".into(), 24, 1, "Fixture".into(), None)));
    }

    /// A fresh directory under the system temporary directory, removed on
    /// drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "toolkit-icons-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&root).unwrap();
            Self(root)
        }

        fn write(&self, path: &str, data: &str) {
            let path = self.0.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, data).unwrap();
        }

        fn lookup(&self, theme: &str) -> Lookup {
            Lookup::in_data_dirs(theme, [self.0.clone()])
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn shell_names_recover_symbolic_glyphs_and_suffixes() {
        let scratch = Scratch::new();
        for name in [
            "preferences-system-notifications",
            "audio-volume-high",
            "network-wired",
            "video-display",
        ] {
            scratch.write(
                &format!("icons/Adwaita/symbolic/status/{name}-symbolic.svg"),
                "svg",
            );
        }
        scratch.write("icons/Adwaita/scalable/apps/plain.svg", "svg");
        let lookup = scratch.lookup("MissingTheme");
        for (name, expected) in [
            ("notifications", "preferences-system-notifications"),
            ("audio-volume-high", "audio-volume-high"),
            ("network-wired-activated", "network-wired"),
            ("user-desktop", "video-display"),
            ("display", "video-display"),
        ] {
            assert_eq!(
                lookup.find(name, 20),
                Some(scratch.0.join(format!(
                    "icons/Adwaita/symbolic/status/{expected}-symbolic.svg"
                ))),
                "{name}"
            );
        }
        assert_eq!(
            lookup.find("plain-symbolic", 20),
            Some(scratch.0.join("icons/Adwaita/scalable/apps/plain.svg"))
        );
        assert!(is_svg(Path::new("icon.SVG")));
        assert!(!is_svg(Path::new("icon.png")));
    }

    #[test]
    fn themed_lookup_inherits_and_chooses_scalable_or_nearest_sized_icons() {
        let scratch = Scratch::new();
        let root = &scratch.0;
        scratch.write(
            "icons/Custom/index.theme",
            "[Icon Theme]\nInherits=hicolor\nDirectories=32x32/apps,64x64/apps\n[32x32/apps]\nSize=32\nType=Fixed\n[64x64/apps]\nSize=64\nType=Fixed\n",
        );
        scratch.write("icons/Custom/32x32/apps/app.png", "png");
        scratch.write("icons/Custom/64x64/apps/app.png", "png");
        scratch.write(
            "icons/hicolor/index.theme",
            "[Icon Theme]\nDirectories=scalable/apps\n[scalable/apps]\nSize=48\nType=Scalable\nMinSize=16\nMaxSize=512\n",
        );
        scratch.write("icons/hicolor/scalable/apps/inherited.svg", "svg");
        scratch.write("applications/app.desktop", "[Desktop Entry]\nIcon=app\n");
        let lookup = scratch.lookup("Custom");
        assert_eq!(
            lookup.find("app", 32),
            Some(root.join("icons/Custom/32x32/apps/app.png"))
        );
        assert_eq!(
            lookup.find("app", 60),
            Some(root.join("icons/Custom/64x64/apps/app.png"))
        );
        assert_eq!(
            lookup.find("inherited", 80),
            Some(root.join("icons/hicolor/scalable/apps/inherited.svg"))
        );
        assert!(lookup.find("../escape", 32).is_none());
        assert_eq!(lookup.application_icon("app"), Some("app".into()));
        assert_eq!(lookup.theme, "Custom");
        assert_eq!(lookup.roots, [root.join("icons")]);
    }

    #[test]
    fn launcher_recovers_missing_desktop_icons_across_theme_layouts() {
        let scratch = Scratch::new();
        let root = &scratch.0;
        // Selected theme missing; a real index can omit package-added sizes.
        scratch.write(
            "icons/hicolor/index.theme",
            "[Icon Theme]\nDirectories=16x16/apps; scalable/apps\n[16x16/apps]\nSize=16\nType=Fixed\n[scalable/apps]\nSize=48\nType=Scalable\nMinSize=16\nMaxSize=512\n",
        );
        scratch.write("icons/hicolor/32x32/apps/firefox.png", "png");
        scratch.write("icons/hicolor/192x192/apps/firefox.png", "png");
        scratch.write(
            "icons/hicolor/512x512/apps/org.mozilla.Thunderbird.png",
            "png",
        );
        scratch.write("icons/hicolor/scalable/apps/foot.svg", "svg");
        scratch.write("icons/hicolor/scalable/apps/rio.svg", "svg");
        scratch.write(
            "icons/Adwaita/index.theme",
            "[Icon Theme]\nInherits=AdwaitaLegacy; hicolor\n",
        );
        scratch.write("icons/AdwaitaLegacy/32x32/legacy/network-wired.png", "png");
        let lookup = scratch.lookup("NotInstalled");
        for (id, icon, expected) in [
            (
                "bssh",
                "network-wired",
                "AdwaitaLegacy/32x32/legacy/network-wired.png",
            ),
            (
                "avahi-discover",
                "network-wired",
                "AdwaitaLegacy/32x32/legacy/network-wired.png",
            ),
            ("firefox", "firefox", "hicolor/32x32/apps/firefox.png"),
            ("footclient", "foot", "hicolor/scalable/apps/foot.svg"),
            ("foot", "foot", "hicolor/scalable/apps/foot.svg"),
            ("rio", "rio", "hicolor/scalable/apps/rio.svg"),
            (
                "org.mozilla.Thunderbird",
                "org.mozilla.Thunderbird",
                "hicolor/512x512/apps/org.mozilla.Thunderbird.png",
            ),
        ] {
            scratch.write(
                &format!("applications/{id}.desktop"),
                &format!("[Desktop Entry]\nIcon={icon}\n"),
            );
            let expected = Some(root.join("icons").join(expected));
            assert_eq!(
                lookup.resolve("", 32, Some(id)),
                expected,
                "{id}: empty source"
            );
            assert_eq!(
                lookup.resolve(
                    "/missing/service/icon.png",
                    32,
                    Some(&format!("{id}.desktop"))
                ),
                expected,
                "{id}: stale path"
            );
            assert_eq!(lookup.resolve(icon, 32, None), expected, "{id}: icon name");
        }
        assert_eq!(
            lookup.find("firefox", 190),
            Some(root.join("icons/hicolor/192x192/apps/firefox.png"))
        );
        assert_eq!(
            lookup.find("firefox.svg", 32),
            Some(root.join("icons/hicolor/32x32/apps/firefox.png"))
        );
        assert_eq!(
            lookup.find("foot.png", 32),
            Some(root.join("icons/hicolor/scalable/apps/foot.svg"))
        );
        assert_eq!(
            lookup.resolve("/old/icons/firefox.png", 32, None),
            Some(root.join("icons/hicolor/32x32/apps/firefox.png"))
        );
    }

    #[test]
    fn pixmaps_absolute_paths_and_nested_desktop_ids_are_recovered() {
        let scratch = Scratch::new();
        let root = std::fs::canonicalize(&scratch.0).unwrap();
        scratch.write("pixmaps/pixmap-app.png", "png");
        scratch.write("private/absolute.svg", "svg");
        scratch.write(
            "applications/vendor/pixmap.desktop",
            "[Desktop Entry]\nIcon=pixmap-app.png\n",
        );
        let absolute = root.join("private/absolute.svg");
        scratch.write(
            "applications/absolute.desktop",
            &format!("[Desktop Entry]\nIcon={}\n", absolute.display()),
        );
        let lookup = Lookup::in_data_dirs("Missing", [root.clone()]);
        assert_eq!(
            lookup.resolve("", 32, Some("vendor-pixmap")),
            Some(root.join("pixmaps/pixmap-app.png"))
        );
        assert_eq!(
            lookup.resolve("", 32, Some("absolute")),
            Some(absolute.clone())
        );
        assert_eq!(
            lookup.resolve(absolute.to_str().unwrap(), 32, None),
            Some(absolute.clone())
        );
        assert_eq!(
            lookup.resolve(&format!("file://{}", absolute.display()), 32, None),
            Some(absolute.clone())
        );
        for name in ["../escape", "../../escape", "/missing.svg"] {
            assert!(lookup.find(name, 32).is_none(), "{name}");
        }
        assert!(lookup.application_icon("../escape").is_none());
        assert!(lookup.application_icon("").is_none());

        // The resolver caches and re-validates.
        let resolver = Resolver::new(lookup);
        assert_eq!(resolver.lookup().theme, "Missing");
        assert_eq!(
            resolver.resolve("", 32, Some("vendor-pixmap")),
            Some(root.join("pixmaps/pixmap-app.png"))
        );
        assert_eq!(resolver.cache.lock().unwrap().len(), 1);
        std::fs::remove_file(root.join("pixmaps/pixmap-app.png")).unwrap();
        assert_eq!(resolver.resolve("", 32, Some("vendor-pixmap")), None);
        assert_eq!(
            resolver.resolve(absolute.to_str().unwrap(), 32, None),
            Some(absolute)
        );
    }

    #[test]
    fn desktop_id_and_category_recover_a_player_without_an_icon_field() {
        let scratch = Scratch::new();
        scratch.write(
            "applications/org.example.player.desktop",
            "[Desktop Entry]\nName=Player\nCategories=AudioVideo;Player;\n",
        );
        scratch.write(
            "icons/Adwaita/symbolic/categories/applications-multimedia-symbolic.svg",
            "svg",
        );
        let lookup = scratch.lookup("Adwaita");
        let expected = scratch
            .0
            .join("icons/Adwaita/symbolic/categories/applications-multimedia-symbolic.svg");
        assert_eq!(
            lookup.resolve("", 32, Some("org.example.player.desktop")),
            Some(expected)
        );
        // A dedicated application icon wins over its category fallback.
        scratch.write("icons/hicolor/scalable/apps/org.example.player.svg", "svg");
        assert_eq!(
            lookup.resolve("", 32, Some("org.example.player.desktop")),
            Some(
                scratch
                    .0
                    .join("icons/hicolor/scalable/apps/org.example.player.svg")
            )
        );
        // Nothing at all resolves to a placeholder when one is installed.
        scratch.write(
            "icons/hicolor/scalable/apps/application-x-executable.svg",
            "svg",
        );
        assert_eq!(
            lookup.resolve("nothing", 32, None),
            Some(
                scratch
                    .0
                    .join("icons/hicolor/scalable/apps/application-x-executable.svg")
            )
        );
    }

    #[test]
    fn xdg_lookup_reads_the_environment_shape() {
        let lookup = Lookup::from_xdg();
        assert!(!lookup.theme.is_empty());
        assert!(lookup.roots.iter().any(|root| root.ends_with(".icons")));
        assert!(lookup.roots.iter().any(|root| root.ends_with("icons")));
        assert_eq!(lookup.pixmaps.len(), lookup.applications.len());
        assert_eq!(list("a, b;c d").collect::<Vec<_>>(), ["a", "b", "c", "d"]);
    }
}
