// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use serde_json::json;

async fn setup() -> (tempfile::TempDir, Db, Uuid) {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::connect(
        dir.path().join("mail.sqlite").to_str().unwrap(),
        dir.path().join("blobs").to_str().unwrap(),
    )
    .await
    .unwrap();
    db.migrate().await.unwrap();
    db.conn
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO accounts(id,email,password) VALUES(1,'user@example.org','unused')",
            [],
        )
        .unwrap();
    let calendar = db::calendar::create_calendar(&db.conn, 1, "Test", None, None, None)
        .await
        .unwrap();
    (dir, db, calendar)
}

#[tokio::test]
async fn calendar_create_keeps_optional_unparseable_dates() {
    let (_dir, db, calendar) = setup().await;
    for (key, mut object) in [
        ("title", json!({"title":"Title only"})),
        (
            "blank",
            json!({"title":"Blank end", "start":"2026-06-01T10:00:00Z", "end":""}),
        ),
        (
            "local",
            json!({"title":"Local start", "start":"2026-06-01T10:00:00", "end":""}),
        ),
    ] {
        object["calendarId"] = json!(calendar);
        let response = event_set(&db, 1, json!({"create":{key:object.clone()}}))
            .await
            .unwrap();
        let id: Uuid = response["created"][key]["id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let events = db::calendar::get_events_by_ids(&db.conn, 1, &[id])
            .await
            .unwrap();
        assert_eq!(events[0].data, object);
        assert_eq!(events[0].start_dt.is_some(), key == "blank");
        assert!(events[0].end_dt.is_none());
    }
}

#[tokio::test]
async fn calendar_update_tolerates_unparseable_stored_dates() {
    let (_dir, db, calendar) = setup().await;
    let response = event_set(
        &db,
        1,
        json!({"create":{"c":{"calendarId":calendar,"title":"Old"}}}),
    )
    .await
    .unwrap();
    let id: Uuid = response["created"]["c"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    db.conn.lock().unwrap().execute("UPDATE calendar_events SET start_dt='not a date', end_dt='2026-06-01T10:00:00' WHERE id=?1", [id.to_string()]).unwrap();
    let response = event_set(
        &db,
        1,
        json!({"update":{id.to_string():{"title":"New","start":"","end":""}}}),
    )
    .await
    .unwrap();
    assert!(response["updated"].get(id.to_string()).is_some());
    let events = db::calendar::get_events_by_ids(&db.conn, 1, &[id])
        .await
        .unwrap();
    assert_eq!(events[0].title.as_deref(), Some("New"));
    assert!(events[0].start_dt.is_none());
    assert!(events[0].end_dt.is_none());
}

#[tokio::test]
async fn calendar_query_ignores_unparseable_date_filters() {
    let (_dir, db, calendar) = setup().await;
    event_set(
        &db,
        1,
        json!({"create":{"c":{"calendarId":calendar,"title":"Title"}}}),
    )
    .await
    .unwrap();
    for value in [
        json!(""),
        json!("not a date"),
        json!("2026-06-01T10:00:00"),
        json!(42),
        json!(null),
    ] {
        let response = event_query(&db, 1, json!({"filter":{"after":value,"before":value}}))
            .await
            .unwrap();
        assert_eq!(response["total"], 1);
    }
}
