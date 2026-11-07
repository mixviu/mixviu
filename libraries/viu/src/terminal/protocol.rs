// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Gold's typed terminal control contract and child refusal relay.
//! Transport authority comes only from VerifiedCommand, never a request body.
//! Application layout and mutations remain owned by Mix.
use ::bus::native_client::VerifiedCommand;
use ::bus::native_session::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
/// Broker provenance is required even when no Unix principal was delivered.
fn stamped(command: &::bus::native_client::IncomingCommand) -> bool {
    let origins: Vec<_> = command
        .headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("broker_origin"))
        .map(|(_, value)| value.as_str())
        .collect();
    origins == ["mesh"]
        || (origins == ["local"]
            && !command.headers.keys().any(|key| {
                ["source_peer", "permissions", "signed_ident"]
                    .iter()
                    .any(|name| key.eq_ignore_ascii_case(name))
            }))
}

/// Replay namespace, never an admission decision. Mesh ingress has no Unix
/// principal, so use the broker's delivered peer and sender instead.
fn caller_key(event: &VerifiedCommand) -> String {
    if let Some(actor) = event.trusted_context() {
        return match &actor.session {
            Some(session) => format!(
                "{}:{:?}:{:?}",
                actor.unix_uid, session.record_id, session.incarnation
            ),
            None => format!(
                "{}:{:?}:{:?}",
                actor.unix_uid, actor.broker_epoch, actor.connection_id
            ),
        };
    }
    let command = event.command();
    format!(
        "bus:{:?}:{:?}:{:?}",
        command.header("broker_origin"),
        command.from,
        command.header("broker_connection")
    )
}

fn caller_epoch(event: &VerifiedCommand, broker_epoch: HexBytes<16>) -> HexBytes<16> {
    if let Some(actor) = event.trusted_context() {
        return actor.connection_id;
    }
    let key = format!("{broker_epoch:?}:{}", caller_key(event));
    let digest = Sha256::digest(key.as_bytes());
    HexBytes(digest[..16].try_into().expect("SHA-256 prefix"))
}

fn caller_label(event: &VerifiedCommand) -> String {
    event
        .trusted_context()
        .map(principal_label)
        .unwrap_or_else(|| {
            let command = event.command();
            format!(
                "Bus {} {}",
                command.header("broker_peer").unwrap_or("local"),
                command.header("broker_connection").unwrap_or(&command.from)
            )
        })
}

pub(crate) fn command_manifest() -> Vec<::bus::CommandDescriptor> {
    use ::bus::CommandDescriptor;
    let mut command_names = vec![CommandDescriptor::new(
        "HELP",
        &[],
        "List all commands this service accepts",
        true,
    )];
    for (name, args, description, read_only) in [
        (
            "term.session",
            &[][..],
            "Read session identity and pane binding diagnostics",
            true,
        ),
        ("term.list", &[][..], "List terminal session state", true),
        ("term.tabs", &[][..], "List tabs and their state", true),
        ("term.panes", &[][..], "List panes and their state", true),
        (
            "term.snapshot",
            &["contents"][..],
            "Read pane screen metadata and optional contents",
            true,
        ),
        (
            "term.type",
            &["text", "foreground_generation"][..],
            "Send text to the target pane",
            false,
        ),
        ("term.tab.new", &[][..], "Open and activate a tab", false),
        ("term.tab.select", &[][..], "Select the target tab", false),
        (
            "term.tab.close",
            &["affected"][..],
            "Close the target tab",
            false,
        ),
        (
            "term.pane.split",
            &["dir"][..],
            "Split the target pane horizontally or vertically",
            false,
        ),
        ("term.pane.select", &[][..], "Select the target pane", false),
        ("term.pane.close", &[][..], "Close the target pane", false),
        (
            "term.operation",
            &["operation_id"][..],
            "Read a retained operation outcome",
            true,
        ),
        (
            "term.execute",
            &["source", "prompt_generation"][..],
            "Execute Mix source at the target shell prompt",
            false,
        ),
        (
            "term.exec.result",
            &["operation_id"][..],
            "Read an execution result",
            true,
        ),
        (
            "term.exec.cancel",
            &["operation_id"][..],
            "Cancel an execution",
            false,
        ),
        (
            "term.task.submit",
            &["source", "argv", "cwd", "env", "timeout_ms"][..],
            "Submit an isolated shell task",
            false,
        ),
        (
            "term.task.result",
            &["operation_id"][..],
            "Read an isolated task result",
            true,
        ),
        (
            "term.task.cancel",
            &["operation_id"][..],
            "Cancel an isolated task",
            false,
        ),
        (
            "term.props.get",
            &["property", "contents"][..],
            "Read pane state or contents through properties",
            true,
        ),
        (
            "term.props.set",
            &["property", "value"][..],
            "Select a pane or send input through properties",
            false,
        ),
    ] {
        let mut descriptor = CommandDescriptor::new(name, args, description, read_only);
        descriptor.args.insert(0, "target".into());
        if !read_only {
            descriptor
                .args
                .extend(["request_id".into(), "request_epoch".into()]);
        }
        command_names.push(descriptor);
    }
    command_names
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub instance_id: HexBytes<16>,
    pub incarnation: HexBytes<16>,
    pub pane_id: DecimalU64,
    pub pane_generation: DecimalU64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub target: Target,
    #[serde(default)]
    pub affected: Vec<Target>,
    #[serde(default)]
    pub request_id: Option<DecimalU64>,
    #[serde(default)]
    pub operation_id: Option<DecimalU64>,
    #[serde(default)]
    pub request_epoch: Option<HexBytes<16>>,
    #[serde(default)]
    pub foreground_generation: Option<DecimalU64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub dir: Option<String>,
    #[serde(default)]
    pub contents: bool,
    #[serde(default)]
    pub property: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    /// Execute family. `source` is the line to run; `prompt_generation` is the
    /// generation the caller believes is at the child's prompt, which is what
    /// turns a stale snapshot into a refusal instead of an execution.
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub prompt_generation: Option<DecimalU64>,
    /// Task family. Relayed as sent — Term never resolves the source/argv
    /// union, so both-or-neither reaches the child's own refusal.
    #[serde(default)]
    pub argv: Option<Vec<String>>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: Vec<(String, String)>,
    #[serde(default)]
    pub timeout_ms: Option<DecimalU64>,
}

impl Request {
    /// Gold bounds the original UTF-8 body before strict decoding. Keep exact
    /// decimal identifiers here; passing them through Mix numbers loses bits.
    pub fn decode(body: &str) -> Result<Self, Reply> {
        if body.len() > 8192 {
            return Err(Reply::error("INVALID_ARGUMENT"));
        }
        serde_json::from_str(body).map_err(|_| Reply::error("INVALID_ARGUMENT"))
    }

    /// Property aliases enter the same owner operation as direct commands.
    pub fn operation<'a>(&mut self, command: &'a str) -> Result<&'a str, Reply> {
        match (command, self.property.as_deref()) {
            ("term.props.get", Some("state")) => Ok("term.session"),
            ("term.props.get", Some("contents")) => {
                self.contents = true;
                Ok("term.snapshot")
            }
            ("term.props.set", Some("selected")) if self.value == Some(Value::Bool(true)) => {
                Ok("term.pane.select")
            }
            ("term.props.set", Some("input")) => {
                self.text = self.value.as_ref().and_then(Value::as_str).map(str::to_owned);
                Ok("term.type")
            }
            _ if !command.starts_with("term.props.") => Ok(command),
            _ => Err(Reply::error("UNSUPPORTED")),
        }
    }
}

/// Retries compare the exact received bytes, including property aliases.
pub fn request_digest(command: &str, body: &str) -> [u8; 32] {
    Sha256::digest(format!("{command}\0{body}").as_bytes()).into()
}

#[derive(Clone)]
pub struct Reply {
    pub rc: u8,
    pub body: String,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FailureCode {
    InvalidArgument,
    NotFound,
    StaleGeneration,
    Conflict,
    Busy,
    Forbidden,
    Unsupported,
    ResourceLimit,
    Disconnected,
    Expired,
    Cancelled,
    UnknownOutcome,
}
#[derive(Serialize)]
struct Failure {
    pub error_code: FailureCode,
    /// Omitted entirely unless a refusal carries something the caller can act
    /// on. Denials stay uniform and detail-free so they reveal nothing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
impl Reply {
    fn ok(value: Value) -> Self {
        Self {
            rc: 0,
            body: value.to_string(),
        }
    }
    pub fn error(code: &'static str) -> Self {
        Self::refuse(code, None)
    }
    /// Only for refusals whose details tell an authorised caller how to make
    /// progress. Never attach details to a denial.
    fn refuse(code: &'static str, details: Option<Value>) -> Self {
        let error_code = match code {
            "INVALID_ARGUMENT" => FailureCode::InvalidArgument,
            "NOT_FOUND" => FailureCode::NotFound,
            "STALE_GENERATION" => FailureCode::StaleGeneration,
            "CONFLICT" => FailureCode::Conflict,
            "BUSY" => FailureCode::Busy,
            "FORBIDDEN" => FailureCode::Forbidden,
            "UNSUPPORTED" => FailureCode::Unsupported,
            "RESOURCE_LIMIT" => FailureCode::ResourceLimit,
            "DISCONNECTED" => FailureCode::Disconnected,
            "EXPIRED" => FailureCode::Expired,
            "CANCELLED" => FailureCode::Cancelled,
            "UNKNOWN_OUTCOME" => FailureCode::UnknownOutcome,
            // A token this table does not know is a typo on the author's side,
            // not a caller's doing, and a request path must not panic over
            // one. Fail closed with the uniform denial and make it loud in a
            // debug build so a test catches the typo rather than production.
            _ => {
                debug_assert!(false, "unregistered Term error token: {code}");
                FailureCode::Forbidden
            }
        };
        Self {
            rc: 10,
            body: serde_json::to_string(&Failure {
                error_code,
                details,
            })
            .expect("owned failure schema"),
        }
    }
}

/// The child's own error code, if its answer was shaped like a refusal at all.
fn refusal_code(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("error_code")?
        .as_str()
        .map(str::to_owned)
}

/// Translate the pane shell's refusal into Term's vocabulary. The child speaks
/// almost the same one; the two that differ are spelled differently for the
/// same meaning, and anything unrecognised is reported as an unknown outcome
/// rather than being flattened into a denial that would read as a policy
/// decision Term never made.
pub fn shell_refusal(body: &str, mutation: bool) -> Reply {
    #[derive(Deserialize)]
    struct Refusal {
        error_code: String,
        #[serde(flatten)]
        rest: serde_json::Map<String, Value>,
    }
    let refusal = serde_json::from_str::<Refusal>(body).ok();
    let code = refusal
        .as_ref()
        .map(|r| r.error_code.clone())
        .unwrap_or_default();
    // The child's refusals CARRY things a caller has to act on: the
    // `operation_id` to ask `term.exec.result` about, and the `reason` that
    // separates "provably did not start, use a new id" from "may have run, go
    // and find out". Relaying only the code discarded both, which left the
    // three-refusal contract unreachable on the only real agent path — the one
    // where every submission goes through Term.
    let relayed = |mapped: &'static str| {
        let details = refusal
            .as_ref()
            .map(|r| &r.rest)
            .filter(|rest| !rest.is_empty());
        match details {
            Some(rest) => Reply::refuse(mapped, Some(Value::Object(rest.clone()))),
            None => Reply::error(mapped),
        }
    };
    match code.as_str() {
        // THREE deliberate exceptions, and nothing else is enumerated here.
        //
        // The map used to list the codes it would relay, which meant every
        // code the child learned to send had to be added to it — and any that
        // was not reached callers as UNKNOWN_OUTCOME, the one answer that says
        // "your request may have run". NOT_FOUND and INVALID_ARGUMENT both sat
        // in that hole. Enumerating the EXCEPTIONS instead makes the default
        // relay: a new child code arrives with its own name, not as a mystery.
        //
        // Two spellings genuinely differ between the surfaces.
        "INVALID_REQUEST" => Reply::error("INVALID_ARGUMENT"),
        // A settled denial: the child looked at the request and said no.
        "REFUSED" => Reply::error("FORBIDDEN"),
        // And a conflict carries TERM's retry contract — byte-identical body —
        // which is Term's rule to state, not the child's.
        "CONFLICT" => Reply::refuse("CONFLICT", Some(mismatch())),
        // Everything in the vocabulary relays under its own name. Denials stay
        // uniform and detail-free; `relayed` already declines to attach an
        // empty body, and a denial carries none.
        other => match vocabulary(other) {
            // Denials stay uniform and detail-free: a refusal that says what it
            // refused is a refusal that leaks. That rule predates this
            // inversion and the inversion must not quietly widen it.
            Some(denial @ ("FORBIDDEN" | "UNSUPPORTED")) => Reply::error(denial),
            Some(known) => relayed(known),
            // Outside the vocabulary, or never shaped like a refusal at all.
            // Flattening these to FORBIDDEN was wrong in both directions: it
            // reads as a policy decision Term never made, and for a transient
            // it tells the caller to stop when it should retry. An
            // unrecognised answer to a mutation is an unknown outcome; to a
            // read, a transport-shaped failure.
            //
            // Logged with the BODY, because this arm is where an answer that
            // was never a refusal disappears without trace. The caller is told
            // "unknown" — which is true — but that is no reason for the
            // operator to be told nothing.
            None if mutation => {
                eprintln!(
                    "term control: unrecognised child answer, reporting unknown outcome: {body}"
                );
                Reply::error("UNKNOWN_OUTCOME")
            }
            None => {
                eprintln!("term control: unrecognised child answer: {body}");
                Reply::error("DISCONNECTED")
            }
        },
    }
}

/// Term's CLOSED error vocabulary, returning the `'static` token so a code can
/// be relayed without an arm of its own.
///
/// This list and `FailureCode` are held to the same set by
/// `every_failure_code_can_be_relayed`, whose exhaustive match fails to build
/// THE TEST TARGET if a variant is added without a token here. That is a
/// `cargo test` build, not a production one — so the guard catches the drift at
/// the gate rather than at `cargo build`, which is where this suite runs
/// anyway. It is the drift that put NOT_FOUND and INVALID_ARGUMENT in the
/// unknown-outcome hole to begin with.
fn vocabulary(code: &str) -> Option<&'static str> {
    const KNOWN: &[&str] = &[
        "INVALID_ARGUMENT",
        "NOT_FOUND",
        "STALE_GENERATION",
        "CONFLICT",
        "BUSY",
        "FORBIDDEN",
        "UNSUPPORTED",
        "RESOURCE_LIMIT",
        "DISCONNECTED",
        "EXPIRED",
        "CANCELLED",
        "UNKNOWN_OUTCOME",
    ];
    KNOWN.iter().copied().find(|known| *known == code)
}

/// The dedupe digest covers the request bytes as sent, deliberately: this
/// recipient does not canonicalise, so two encodings of the same object are two
/// different payloads. A retry MUST replay the body byte for byte, and the
/// refusal says so rather than leaving a caller to guess why its "identical"
/// retry conflicted.
fn mismatch() -> Value {
    json!({"reason":"request_mismatch","retry_requires":"byte_identical_body"})
}
/// The high-water mark is advanced before the command runs, so an ID can be retired
/// without its outcome ever being recorded (BROKER-022: a committed mutation
/// must never re-execute, even when its result was lost). An actor holding only
/// Input or Terminate cannot read the mark back through term.session, so the
/// refusal carries the floor it has to climb past to make progress again.
fn retired(high_water: u64) -> Value {
    json!({"reason":"retired_request_id","request_high_water":DecimalU64(high_water)})
}
/// Command classification for dispatch, independent of the caller's capabilities.
pub(crate) fn capability_of(command_name: &str, contents: bool) -> Option<Capability> {
    Some(match command_name {
        "term.macros.run" | "term.macros.stop" | "term.macros.reload" | "term.macros.new" => {
            Capability::Execute
        }
        "term.macros.list" | "term.macros.status" | "term.macros.output" => Capability::ReadState,
        "term.session" | "term.list" | "term.tabs" | "term.panes" | "term.operation" => {
            Capability::ReadState
        }
        "term.snapshot" if !contents => Capability::ReadState,
        "term.snapshot" => Capability::ReadContents,
        "term.type" => Capability::Input,
        "term.tab.new" | "term.tab.select" | "term.pane.split" | "term.pane.select" => {
            Capability::ManageLayout
        }
        "term.tab.close" | "term.pane.close" => Capability::Terminate,
        // One capability for the whole execute family. Asking what an execution
        // did is asking about an execution.
        "term.execute" | "term.exec.result" | "term.exec.cancel" => Capability::Execute,
        "term.task.submit" | "term.task.result" | "term.task.cancel" => Capability::Execute,
        _ => return None,
    })
}

/// The name the PANE announces for a forwarded submission. Built from Term's
/// own broker-stamped actor context and nothing else: a caller has no field
/// through which to supply a name, because a name a caller chose would let one
/// agent announce itself as another.
fn principal_label(actor: &BrokerPrincipal) -> String {
    match &actor.session {
        Some(s) => {
            let record: String = s
                .record_id
                .0
                .iter()
                .take(4)
                .map(|byte| format!("{byte:02x}"))
                .collect();
            format!("{:?} {}", s.role, record)
        }
        None => format!("uid {} pid {}", actor.unix_uid, actor.peer_pid),
    }
}

/// Target validation is independent of the caller.
pub(crate) fn target_matches(parent: &SessionRecord, target: &Target) -> bool {
    parent.instance_id == target.instance_id
        && parent.incarnation == target.incarnation
        && target.pane_generation.0 > 0
}

#[cfg(test)]
mod vocabulary_tests {
    use super::*;

    /// The relay map and Term's error enum must be the same set.
    ///
    /// The match below has no wildcard, so adding a `FailureCode` variant
    /// without giving it a token here fails to compile THIS TEST — the gate
    /// stops, rather than a code silently reaching callers as UNKNOWN_OUTCOME.
    /// That silence is exactly what happened to NOT_FOUND and INVALID_ARGUMENT.
    #[test]
    fn every_failure_code_can_be_relayed() {
        fn token(code: &FailureCode) -> &'static str {
            match code {
                FailureCode::InvalidArgument => "INVALID_ARGUMENT",
                FailureCode::NotFound => "NOT_FOUND",
                FailureCode::StaleGeneration => "STALE_GENERATION",
                FailureCode::Conflict => "CONFLICT",
                FailureCode::Busy => "BUSY",
                FailureCode::Forbidden => "FORBIDDEN",
                FailureCode::Unsupported => "UNSUPPORTED",
                FailureCode::ResourceLimit => "RESOURCE_LIMIT",
                FailureCode::Disconnected => "DISCONNECTED",
                FailureCode::Expired => "EXPIRED",
                FailureCode::Cancelled => "CANCELLED",
                FailureCode::UnknownOutcome => "UNKNOWN_OUTCOME",
            }
        }
        for code in [
            FailureCode::InvalidArgument,
            FailureCode::NotFound,
            FailureCode::StaleGeneration,
            FailureCode::Conflict,
            FailureCode::Busy,
            FailureCode::Forbidden,
            FailureCode::Unsupported,
            FailureCode::ResourceLimit,
            FailureCode::Disconnected,
            FailureCode::Expired,
            FailureCode::Cancelled,
            FailureCode::UnknownOutcome,
        ] {
            let token = token(&code);
            assert_eq!(vocabulary(token), Some(token), "{token} is not relayable");
            // And the token is the wire spelling the enum itself serialises to,
            // so the two halves cannot disagree about what a code is called.
            let body = Reply::error(vocabulary(token).expect("in vocabulary")).body;
            assert_eq!(
                serde_json::from_str::<Value>(&body).expect("a refusal body")["error_code"],
                json!(token)
            );
        }
    }

    /// A child code Term has never heard of must NOT relay.
    #[test]
    fn an_unknown_code_is_not_in_the_vocabulary() {
        assert_eq!(vocabulary("TEAPOT"), None);
        assert_eq!(vocabulary(""), None);
        // Spelling differences are handled as explicit exceptions, not by the
        // vocabulary: these are the child's words, not Term's.
        assert_eq!(vocabulary("INVALID_REQUEST"), None);
        assert_eq!(vocabulary("REFUSED"), None);
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;

    fn body() -> Value {
        json!({"target":{"instance_id":"11".repeat(16),"incarnation":"22".repeat(16),"pane_id":u64::MAX.to_string(),"pane_generation":"9007199254740993"},"request_id":u64::MAX.to_string()})
    }

    #[test]
    fn exact_u64_survives_decode_and_body_cannot_assert_authority() {
        let mut value = body();
        let request = Request::decode(&value.to_string()).ok().unwrap();
        assert_eq!(request.target.pane_id.0, u64::MAX);
        assert_eq!(request.target.pane_generation.0, 9_007_199_254_740_993);
        assert_eq!(request.request_id.unwrap().0, u64::MAX);
        for invalid in [json!(u64::MAX), json!("01"), json!("18446744073709551616")] {
            value["target"]["pane_id"] = invalid;
            assert!(Request::decode(&value.to_string()).is_err());
        }
        for field in ["actor", "broker_origin", "permissions", "signed_ident"] {
            let mut forged = body();
            forged[field] = json!("local");
            assert!(Request::decode(&forged.to_string()).is_err());
        }
    }

    #[test]
    fn multibyte_body_budget_and_retry_identity_use_original_bytes() {
        let mut value = body();
        value["text"] = json!("é".repeat(4096));
        assert!(Request::decode(&value.to_string()).is_err());
        let plain = body().to_string();
        let spaced = serde_json::to_string_pretty(&body()).unwrap();
        assert!(Request::decode(&plain).is_ok());
        assert!(Request::decode(&spaced).is_ok());
        assert_ne!(request_digest("term.type", &plain), request_digest("term.type", &spaced));
        assert_ne!(request_digest("term.type", &plain), request_digest("term.snapshot", &plain));
    }

    #[test]
    fn property_aliases_share_direct_operations_and_refuse_unsupported_values() {
        let mut value = body();
        value["property"] = json!("contents");
        let mut request = Request::decode(&value.to_string()).ok().unwrap();
        assert_eq!(request.operation("term.props.get").ok(), Some("term.snapshot"));
        assert!(request.contents);
        value["property"] = json!("input");
        value["value"] = json!("hello");
        let mut request = Request::decode(&value.to_string()).ok().unwrap();
        assert_eq!(request.operation("term.props.set").ok(), Some("term.type"));
        assert_eq!(request.text.as_deref(), Some("hello"));
        value["property"] = json!("selected");
        for invalid in [json!(false), json!("true"), Value::Null] {
            value["value"] = invalid;
            let mut request = Request::decode(&value.to_string()).ok().unwrap();
            assert!(request.operation("term.props.set").is_err());
        }
    }

    #[test]
    fn child_failures_keep_recovery_details_but_denials_are_uniform() {
        let answer = shell_refusal(r#"{"error_code":"BUSY","operation_id":"18446744073709551615","reason":"running"}"#, true);
        let value: Value = serde_json::from_str(&answer.body).unwrap();
        assert_eq!(value["details"]["operation_id"], u64::MAX.to_string());
        for code in ["FORBIDDEN", "UNSUPPORTED", "REFUSED"] {
            let reply = shell_refusal(&json!({"error_code":code,"secret":"must not relay"}).to_string(), true);
            let value: Value = serde_json::from_str(&reply.body).unwrap();
            assert!(value.get("details").is_none());
            assert!(!reply.body.contains("secret"));
        }
        assert_eq!(serde_json::from_str::<Value>(&shell_refusal("not json", true).body).unwrap()["error_code"], "UNKNOWN_OUTCOME");
        assert_eq!(serde_json::from_str::<Value>(&shell_refusal("not json", false).body).unwrap()["error_code"], "DISCONNECTED");
    }
}
