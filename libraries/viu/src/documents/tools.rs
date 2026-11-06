// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared bounded mirror search and source-tagged diagnostic operations.
use super::*;
use editor_model::{
    diag::{DiagItem, Severity},
    highlight::ResultTag,
};
use sha2::{Digest, Sha256};
fn contents(d: &Document) -> String {
    let mut s = String::new();
    d.mirror.text().read(0..d.mirror.text().len(), &mut s);
    s
}
pub(super) fn find(d: &mut Document, a: &Value, ids: &mut OpIdGen) -> Result<Value, String> {
    let pattern = text_arg(a, "pattern")?;
    if pattern.is_empty() {
        return Err("pattern must not be empty".into());
    }
    let pattern = if a["regex"] == true {
        pattern.to_owned()
    } else {
        regex::escape(pattern)
    };
    let regex = regex::RegexBuilder::new(&pattern)
        .multi_line(true)
        .case_insensitive(a["case"] == false)
        .size_limit(edit::limits::REGEX_SIZE_LIMIT)
        .build()
        .map_err(|e| format!("bad_regex: {e}"))?;
    let text = contents(d);
    let kind = a["kind"].as_str().unwrap_or("next");
    let lo = d.model.sel.anchor.min(d.model.sel.head);
    let hi = d.model.sel.anchor.max(d.model.sel.head);
    let replacing=matches!(kind,"replace"|"replace-all");
    let mut matches=Vec::new();let mut count=0;let mut first=None;let mut last=None;let mut selected=None;
    d.matches.clear();
    for m in regex.find_iter(&text) {
        let r=m.range();count+=1;
        if first.is_none(){first=Some(r.clone());}last=Some(r.clone());
        if d.matches.len()<1000 {d.matches.push(r.clone());}
        if replacing && (kind=="replace-all" || r==(lo..hi)) {matches.push(r.clone());if kind=="replace-all" && matches.len()>edit::limits::MAX_OPS_PER_TXN {return Err("Too large for one undoable transaction (over 10000 matches)".into());}}
        if kind=="prev" {if r.end<=lo && r!=(lo..hi){selected=Some(r);}}
        else if selected.is_none() && r.start>=hi && r!=(lo..hi){selected=Some(r);}
    }
    let selected=selected.or(if kind=="prev" {last}else{first});
    let mut items = Vec::new();
    if kind == "replace-all" || (kind == "replace" && hi > lo && matches.contains(&(lo..hi))) {
        if !d.mirror.is_idle() {
            return Err("Wait for pending edits before replacing".into());
        }
        if matches.len() > edit::limits::MAX_OPS_PER_TXN {
            return Err("Too large for one undoable transaction (over 10000 matches)".into());
        }
        let replacement = text_arg(a, "replacement")?;
        let mut bytes = 0;
        for r in &matches {
            if kind == "replace" && *r != (lo..hi) {
                continue;
            }
            let replacement = if a["expand"] == true {
                let capture = regex
                    .captures_at(&text, r.start)
                    .ok_or("capture disappeared")?;
                let mut s = String::new();
                capture.expand(replacement, &mut s);
                s
            } else {
                replacement.to_owned()
            };
            bytes += replacement.len();
            if bytes > edit::limits::MAX_REQUEST_TEXT_BYTES {
                return Err(
                    "Too large for one undoable transaction (over 1 MiB replacement text)".into(),
                );
            }
            items.push((r.clone(), replacement));
        }
    }
    let n = items.len();
    let mut op_id = None;
    if !items.is_empty() {
        let intent = a["caller"]
            .as_str()
            .map_or_else(|| Intent::ui(d.identity), |c| Intent::bus(d.identity, c));
        let (step, id) = d.mirror.server_op(
            ServerOp::ApplyAt {
                items,
                expect_rev: d.mirror.rev(),
            },
            intent,
            ids,
        );
        d.step(step);
        op_id = Some(id);
    } else if let Some(range) = selected {
        d.model.sel = Selection {
            anchor: range.start,
            head: range.end,
        };
        d.model.preferred_cells = None;
    }
    Ok(json!({"found":count>0,"matches":count,"replaced":n,"op_id":op_id}))
}
pub(super) fn diagnostics(d: &mut Document, a: &Value) -> Result<Value, String> {
    if serde_json::to_vec(a).map_err(|e| e.to_string())?.len() > 64 * 1024 {
        return Err("diagnostics exceed 64 KiB".into());
    }
    let source = text_arg(a, "source")?;
    if source == "lint"
        || source.is_empty()
        || source.len() > 32
        || !source.as_bytes()[0].is_ascii_lowercase()
        || !source
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("invalid or reserved diagnostic source".into());
    }
    let rows = a["diagnostics"]
        .as_array()
        .ok_or("diagnostics list required")?;
    if rows.len() > 500 {
        return Err("at most 500 diagnostics".into());
    }
    let mut items = Vec::new();
    for row in rows {
        let line = crate::widgets::whole(&row["line"])
            .filter(|n| *n > 0)
            .ok_or("line must be positive")? as usize;
        let column = if row["col"].is_null() {
            None
        } else {
            Some(
                crate::widgets::whole(&row["col"])
                    .filter(|n| *n > 0)
                    .ok_or("column must be positive")? as usize,
            )
        };
        let severity = match row["severity"].as_str() {
            Some("error") => Severity::Error,
            Some("warning") => Severity::Warning,
            Some("note") => Severity::Note,
            _ => return Err("invalid severity".into()),
        };
        items.push(DiagItem {
            line,
            column,
            severity,
            code: text_arg(row, "code")?.into(),
            message: text_arg(row, "message")?.into(),
            hint: None,
        });
    }
    let text = contents(d);
    let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
    let stale = if let Some(want) = a["digest"].as_str() {
        if want.len() != 64
            || !want
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("digest must be lower-case sha256".into());
        }
        want != digest
    } else {
        false
    };
    let tag = ResultTag {
        epoch: d.mirror.epoch().into(),
        buffer: d.mirror.buffer().into(),
        view_gen: d.mirror.view_gen(),
        language: d.mirror.meta().language.clone(),
        cfg: 0,
    };
    d.diagnostics
        .accept_items(
            source,
            &tag,
            tag.clone(),
            &text,
            if stale { &[] } else { &items },
            &[],
        )
        .map_err(|e| format!("{e:?}"))?;
    Ok(
        json!({"shown":d.diagnostics.items().iter().filter(|i|i.source==source).count(),"stale":stale}),
    )
}
pub(super) fn problems(d: &Document) -> Value {
    json!(d.diagnostics.items().iter().map(|item|{let p=d.mirror.text().point(item.range.start);json!({"line":p.line,"col":p.col,"severity":match item.severity{Severity::Error=>"error",Severity::Warning=>"warning",Severity::Note=>"note"},"code":item.code,"message":item.message,"source":item.source})}).collect::<Vec<_>>())
}
pub(super) fn render_diagnostics(d: &Document) -> Vec<editor_model::diag::Diagnostic> {
    d.diagnostics.items().to_vec()
}
