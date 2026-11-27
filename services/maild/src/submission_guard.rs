//! Shared, durable submission admission. Reservations are deliberately not
//! refunded: failures and retries consume quota, so crashes cannot reopen it.
use crate::db::Db;
use anyhow::Result;

#[derive(Debug)]
pub enum Refusal {
    Recipient,
    Rate,
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Recipient => "recipientNotAllowed",
            Self::Rate => "sendRateExceeded",
        })
    }
}
impl std::error::Error for Refusal {}

// Glob literals, * and ?, ASCII case-insensitive, anchored at both ends.
fn matches(pattern: &str, address: &str) -> bool {
    let p = pattern.to_ascii_lowercase();
    let a = address.to_ascii_lowercase();
    let (p, a) = (p.as_bytes(), a.as_bytes());
    let (mut i, mut j, mut star, mut retry) = (0, 0, None, 0);
    while j < a.len() {
        if i < p.len() && (p[i] == b'?' || p[i] == a[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == b'*' {
            star = Some(i);
            i += 1;
            retry = j;
        } else if let Some(s) = star {
            retry += 1;
            j = retry;
            i = s + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == b'*' {
        i += 1;
    }
    i == p.len()
}

#[cfg(test)]
fn reserve(
    conn: &mut rusqlite::Connection,
    account: i32,
    recipients: &[String],
    now: i64,
) -> Result<()> {
    reserve_messages(conn, account, recipients, now, 1)
}

fn reserve_messages(
    conn: &mut rusqlite::Connection,
    account: i32,
    recipients: &[String],
    now: i64,
    messages: usize,
) -> Result<()> {
    let messages = i64::try_from(messages)?;
    anyhow::ensure!(
        messages > 0,
        "reservation must contain at least one message"
    );
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let (allowed, rate): (Option<String>, Option<i64>) = tx.query_row(
        "SELECT allowed_recipients, send_rate FROM accounts WHERE id=?1",
        [account],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if let Some(allowed) = allowed {
        let patterns: Vec<String> = serde_json::from_str(&allowed)?;
        if recipients
            .iter()
            .any(|r| !patterns.iter().any(|p| matches(p, r)))
        {
            return Err(Refusal::Recipient.into());
        }
    }
    // Keep future timestamps counted too (clock rollback fails closed).
    tx.execute(
        "DELETE FROM submission_rate WHERE account_id=?1 AND reserved_at<=?2",
        rusqlite::params![account, now - 3600],
    )?;
    let used: i64 = tx.query_row(
        "SELECT COUNT(*) FROM submission_rate WHERE account_id=?1",
        [account],
        |r| r.get(0),
    )?;
    if rate.is_some_and(|r| used.checked_add(messages).is_none_or(|total| total > r)) {
        return Err(Refusal::Rate.into());
    }
    for _ in 0..messages {
        tx.execute(
            "INSERT INTO submission_rate(account_id,reserved_at) VALUES (?1,?2)",
            rusqlite::params![account, now],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub async fn admit(db: &Db, account: i32, recipients: &[String]) -> Result<()> {
    admit_messages(db, account, recipients, 1).await
}

async fn admit_messages(
    db: &Db,
    account: i32,
    recipients: &[String],
    messages: usize,
) -> Result<()> {
    let conn = db.conn.clone();
    let recipients = recipients.to_vec();
    tokio::task::spawn_blocking(move || {
        let mut conn = conn
            .lock()
            .map_err(|_| anyhow::anyhow!("submission DB lock poisoned"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        reserve_messages(&mut conn, account, &recipients, now, messages)
    })
    .await?
}

/// Account-authored automatic mail must not reopen a doorway. Suppression,
/// including storage failure, never turns the original delivery into a bounce.
pub(crate) async fn generated_allowed(db: &Db, account: i32, recipients: &[String]) -> bool {
    generated_messages_allowed(db, account, recipients, 1).await
}

/// Reserve the entire fan-out atomically: each separately queued message costs
/// one slot. Ordinary multi-recipient submissions and single replies cost one.
pub(crate) async fn generated_messages_allowed(
    db: &Db,
    account: i32,
    recipients: &[String],
    messages: usize,
) -> bool {
    match admit_messages(db, account, recipients, messages).await {
        Ok(()) => true,
        Err(error) => {
            tracing::warn!(account, %error, "Account-generated outbound mail suppressed by submission guard");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_envelope_and_durable_window() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut c = rusqlite::Connection::open(tmp.path()).unwrap();
        c.execute_batch(crate::db::SCHEMA).unwrap();
        c.execute("INSERT INTO accounts(id,email,password,allowed_recipients,send_rate) VALUES (1,'cast@example.com','x','[\"mc@*.example\"]',1)", []).unwrap();
        let good = vec!["mc@door.example".into()];
        assert!(
            reserve(
                &mut c,
                1,
                &["mc@door.example".into(), "other@example.com".into()],
                10000
            )
            .unwrap_err()
            .is::<Refusal>()
        );
        reserve(&mut c, 1, &good, 10000).unwrap();
        drop(c);
        let mut c = rusqlite::Connection::open(tmp.path()).unwrap();
        assert!(
            reserve(&mut c, 1, &good, 13599)
                .unwrap_err()
                .is::<Refusal>()
        );
        reserve(&mut c, 1, &good, 13600).unwrap();
    }
}
