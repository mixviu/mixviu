// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compatibility keys at lookup boundaries; live window IDs stay authored.

pub fn canonical(id: &str) -> String {
    // gallery-reference owns mixviu.gallery.<mode>; it is not the Mix gallery.
    // Only desktop/icon filename extensions belong to the base gallery alias.
    if let Some(tail) = id.strip_prefix("mixviu.gallery.")
        && !matches!(tail, "desktop" | "svg" | "png")
    {
        return id.into();
    }
    if let Some(rest) = id.strip_prefix("dev.mixos.") {
        return format!("dev.mix.{rest}");
    }
    if let Some(rest) = id.strip_prefix("mixviu.") {
        for app in [
            "term-mix",
            "gallery-mix",
            "ced",
            "term",
            "deck",
            "bustap",
            "gallery",
        ] {
            if let Some(tail) = rest.strip_prefix(app)
                && (tail.is_empty() || tail.starts_with('.') || tail.starts_with("-symbolic"))
            {
                return format!("dev.mix.{}{tail}", app.strip_suffix("-mix").unwrap_or(app));
            }
        }
    }
    id.into()
}

/// Canonical wins if both generations exist; old references remain readable.
pub fn candidates(id: &str) -> Vec<String> {
    let key = canonical(id);
    let mut out = vec![key.clone()];
    if let Some(rest) = key.strip_prefix("dev.mix.") {
        out.push(format!("dev.mixos.{rest}"));
        let mixed = format!("mixviu.{rest}");
        if canonical(&mixed) == key {
            out.push(mixed);
        }
        for app in ["term", "gallery"] {
            if let Some(tail) = rest.strip_prefix(app)
                && (tail.is_empty() || tail.starts_with('.') || tail.starts_with("-symbolic"))
            {
                out.push(format!("mixviu.{app}-mix{tail}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_keys_preserve_secondary_windows_and_vendor_ids() {
        let cases: Vec<serde_json::Value> = strict::from_str(include_str!(
            "../../../engines/viud/tests/fixtures/apps/legacy-ids.conf.mix"
        ))
        .unwrap();
        for case in cases {
            let old = case["old"].as_str().unwrap();
            let current = case["current"].as_str().unwrap();
            assert_eq!(canonical(old), current);
            assert!(candidates(current).contains(&old.to_owned()));
        }
        for app in ["ced", "term", "deck", "bustap", "gallery"] {
            let new = format!("dev.mix.{app}");
            assert_eq!(canonical(&format!("dev.mixos.{app}")), new);
            assert_eq!(canonical(&format!("mixviu.{app}")), new);
            assert_eq!(candidates(&new)[0], new);
        }
        assert_eq!(canonical("mixviu.term-mix"), "dev.mix.term");
        assert_eq!(
            canonical("dev.mixos.email.compose-2"),
            "dev.mix.email.compose-2"
        );
        assert_eq!(canonical("mixviu.ced-symbolic"), "dev.mix.ced-symbolic");
        for id in [
            "org.example.app",
            "mixviu.terminal",
            "mixviu.term-mixer",
            "mixviu.gallery.native",
            "mixviu.gallery.light",
            "dev.mixos",
            "/icons/dev.mixos.ced.svg",
        ] {
            assert_eq!(canonical(id), id);
            assert_eq!(candidates(id), [id]);
        }
    }
}
