// SPDX-License-Identifier: MIT OR Apache-2.0
//! Whole-source admission at build/install and explicit scene write boundaries.
pub fn validate_source(source: &str) -> Result<(), String> {
    let document = design::parse_design_source(design::SourceIdentity::new(design::source_digest(source)), source).map_err(|e| e.to_string())?;
    validate_document(&document)
}

pub fn validate_document(document: &design::DesignSourceDocument) -> Result<(), String> {
    use std::{collections::BTreeMap, sync::{Mutex, OnceLock}};
    static RESULTS: OnceLock<Mutex<BTreeMap<String, Result<(), String>>>> = OnceLock::new();
    let results = RESULTS.get_or_init(Default::default);
    let digest = design::document_digest(document);
    if let Some(result) = results.lock().unwrap_or_else(|e| e.into_inner()).get(&digest).cloned() {
        return result;
    }
    let result = match design::compile_design(document, design::DesignContext::default()) {
        design::DesignCompileResult::Success(_) => Ok(()),
        design::DesignCompileResult::Fatal(failure) => Err(format!("{:?}", failure.diagnostics)),
    };
    let mut cache = results.lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() >= 16 { cache.clear(); }
    cache.insert(digest, result.clone());
    result
}

/// The same source used by the desk, without admitting a fallback over a bad
/// file. The source read and compiled here is one immutable theme snapshot.
pub fn validate_desk_theme() -> Result<(), String> {
    let theme = toolkit::Theme::load();
    if let Some(error) = theme.load_error() { return Err(error.into()); }
    validate_document(&theme.source_document())
}
