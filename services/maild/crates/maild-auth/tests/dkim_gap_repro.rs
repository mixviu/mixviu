// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Reproduce apparent missing DKIM results without publishing private mail.
//! For the ignored test, set DKIM_GAP_FIXTURES to a directory containing three
//! unmodified stored .eml blobs. Only the leading A-R blocks are removed before
//! exercising the verifier's trim -> parse -> verify_dkim -> A-R builder path.

use mail_auth::{
    AuthenticatedMessage, AuthenticationResults, DkimResult, MessageAuthenticator, Parameters,
    ResolverCache, Txt,
};
use maild_auth::ar_trim::trim_forged_authentication_results;
use std::{borrow::Borrow, hash::Hash, time::Instant};

// Use mail-auth's TXT cache injection point to make every lookup return an
// error locally. Resolver construction performs no network I/O.
struct OfflineTxt;

impl ResolverCache<Box<str>, Txt> for OfflineTxt {
    fn get<Q>(&self, _: &Q) -> Option<Txt>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        Some(Txt::Error(mail_auth::Error::Dns(
            mail_auth::DnsError::InvalidRecordType,
        )))
    }

    fn remove<Q>(&self, _: &Q) -> Option<Txt>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        None
    }

    fn insert(&self, _: Box<str>, _: Txt, _: Instant) {
        panic!("offline TXT cache must satisfy every lookup");
    }
}

fn strip_leading_ar(mut raw: &[u8]) -> &[u8] {
    const NAME: &[u8] = b"Authentication-Results:";
    while raw.len() >= NAME.len() && raw[..NAME.len()].eq_ignore_ascii_case(NAME) {
        let mut end = 0;
        loop {
            end += raw[end..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(raw.len() - end, |n| n + 1);
            if end == raw.len() || !matches!(raw[end], b' ' | b'\t') {
                break;
            }
        }
        raw = &raw[end..];
    }
    raw
}

async fn check_outer(label: &str, stored: &[u8]) {
    let raw = strip_leading_ar(stored);
    let stored_parsed = AuthenticatedMessage::parse(stored).expect("stored message parses");
    let original = AuthenticatedMessage::parse(raw).expect("input message parses");
    let (trimmed, removed) = trim_forged_authentication_results(raw, "maild.example.org");
    let parsed = AuthenticatedMessage::parse(&trimmed).expect("trimmed message parses");
    assert_eq!(removed, 0);
    assert_eq!(trimmed, raw, "A-R trim must preserve input bytes");
    assert_eq!(stored_parsed.raw_body(), parsed.raw_body());
    assert_eq!(original.raw_body(), parsed.raw_body());
    assert!(stored_parsed.dkim_headers.is_empty());
    assert!(original.dkim_headers.is_empty());
    assert!(parsed.dkim_headers.is_empty());

    let signature = b"DKIM-Signature:";
    let nested_offset = raw
        .windows(signature.len())
        .position(|s| s.eq_ignore_ascii_case(signature))
        .expect("fixture contains signature text");
    assert!(nested_offset >= parsed.body_offset());
    let auth = MessageAuthenticator::new_cloudflare().unwrap();
    let outputs = auth
        .verify_dkim(Parameters::new(&parsed).with_txt_cache(&OfflineTxt))
        .await;
    println!(
        "{label}: body_offset={} signature_offset={nested_offset} DKIM outputs={} results={:?}",
        parsed.body_offset(),
        outputs.len(),
        outputs.iter().map(|o| o.result()).collect::<Vec<_>>()
    );
    assert!(outputs.is_empty());
    let ar = AuthenticationResults::new("maild.example.org")
        .with_dkim_results(&outputs, parsed.from())
        .to_string();
    assert!(!ar.contains("dkim="));

    // Positive control: parsing at the nested signature recognises it and
    // produces a result. It does not authenticate the enclosing bounce.
    let nested = AuthenticatedMessage::parse(&raw[nested_offset..]).unwrap();
    let nested_outputs = auth
        .verify_dkim(Parameters::new(&nested).with_txt_cache(&OfflineTxt))
        .await;
    println!(
        "{label}: nested DKIM outputs={} results={:?}",
        nested_outputs.len(),
        nested_outputs
            .iter()
            .map(|o| o.result())
            .collect::<Vec<_>>()
    );
    assert!(!nested_outputs.is_empty());
    assert_eq!(nested_outputs.len(), nested.dkim_headers.len());
}

#[tokio::test]
async fn attached_signature_does_not_authenticate_bounce() {
    let stored = b"Authentication-Results: maild.example.org;\r\n\
        \tdmarc=fail\r\n\
        From: Mail Delivery System <postmaster@example.org>\r\n\
        Content-Type: multipart/report; boundary=bounce\r\n\r\n\
        --bounce\r\nContent-Type: message/rfc822\r\n\r\n\
        DKIM-Signature: v=1; a=rsa-sha256; d=example.org; s=mail; h=from; bh=AAAA; b=AAAA\r\n\
        From: sender@example.org\r\n\r\nbody\r\n--bounce--\r\n";
    check_outer("synthetic bounce", stored).await;
}

// mail-auth 0.11 requires d, s, b, bh and h before a signature reaches
// dkim_headers, so this fixture carries every tag with a bogus hash: the
// folded, mixed-case header is still recognised and fails as neutral.
#[tokio::test]
async fn malformed_folded_mixed_case_signature_still_produces_output() {
    let raw = b"dKiM-sIgNaTuRe: v=1; a=rsa-sha256;\r\n\
        \td=example.org; s=mail; h=from; bh=AAAA; b=AAAA\r\n\
        From: sender@example.org\r\n\r\nbody\r\n";
    let parsed = AuthenticatedMessage::parse(raw).unwrap();
    assert_eq!(parsed.dkim_headers.len(), 1);
    let auth = MessageAuthenticator::new_cloudflare().unwrap();
    let outputs = auth
        .verify_dkim(Parameters::new(&parsed).with_txt_cache(&OfflineTxt))
        .await;
    assert_eq!(outputs.len(), 1);
    println!(
        "malformed top-level: outputs={} {:?}",
        outputs.len(),
        outputs[0].result()
    );
    assert!(matches!(outputs[0].result(), DkimResult::Neutral(_)));
    assert!(
        AuthenticationResults::new("maild.example.org")
            .with_dkim_results(&outputs, parsed.from())
            .to_string()
            .contains("dkim=neutral")
    );
}

// mail-auth 0.11 turns a signature missing required tags into a
// MissingParameters parse error and verify_dkim reports nothing for it (it
// surfaces only Error::Dkim parse errors). Pin that upstream behaviour, then
// check maild restores the RFC 8601 `dkim=neutral` result 0.7 produced.
#[tokio::test]
async fn signature_missing_required_tags_is_reported_neutral() {
    let raw = b"DKIM-Signature: v=1; d=example.org; s=mail\r\n\
        From: sender@example.org\r\n\r\nbody\r\n";
    let parsed = AuthenticatedMessage::parse(raw).unwrap();
    assert!(parsed.dkim_headers.is_empty());
    assert!(parsed.has_dkim_errors);
    assert!(
        parsed
            .errors
            .iter()
            .any(|h| matches!(h.header, mail_auth::Error::MissingParameters))
    );
    let auth = MessageAuthenticator::new_cloudflare().unwrap();
    let mut outputs = auth
        .verify_dkim(Parameters::new(&parsed).with_txt_cache(&OfflineTxt))
        .await;
    assert!(outputs.is_empty(), "upstream behaviour changed; revisit");

    maild_auth::mapping::surface_dkim_parse_errors(&parsed, &mut outputs);
    assert_eq!(outputs.len(), 1);
    assert!(matches!(outputs[0].result(), DkimResult::Neutral(_)));
    assert!(
        AuthenticationResults::new("maild.example.org")
            .with_dkim_results(&outputs, parsed.from())
            .to_string()
            .contains("dkim=neutral")
    );
}

// A signature mail-auth already reports (an Error::Dkim parse error) must not
// be reported twice.
#[tokio::test]
async fn dkim_parse_errors_are_not_reported_twice() {
    let raw = b"DKIM-Signature: v=2; d=example.org; s=mail\r\n\
        From: sender@example.org\r\n\r\nbody\r\n";
    let parsed = AuthenticatedMessage::parse(raw).unwrap();
    let auth = MessageAuthenticator::new_cloudflare().unwrap();
    let mut outputs = auth
        .verify_dkim(Parameters::new(&parsed).with_txt_cache(&OfflineTxt))
        .await;
    assert_eq!(outputs.len(), 1);
    maild_auth::mapping::surface_dkim_parse_errors(&parsed, &mut outputs);
    assert_eq!(outputs.len(), 1);
}

#[tokio::test]
#[ignore = "requires three private .eml files in DKIM_GAP_FIXTURES"]
async fn stored_dkim_gap_samples() {
    let dir = std::env::var_os("DKIM_GAP_FIXTURES").expect("set DKIM_GAP_FIXTURES");
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "eml"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 3, "supply exactly three private fixtures");
    for path in paths {
        check_outer(
            path.file_name().unwrap().to_str().unwrap(),
            &std::fs::read(&path).unwrap(),
        )
        .await;
    }
}
