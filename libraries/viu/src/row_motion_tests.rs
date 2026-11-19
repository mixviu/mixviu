use super::*;

#[test]
fn row_identity_survives_head_edits_sort_and_duplicate_occurrences() {
    let rows = vec![json!(["a"]), json!(["b"]), json!(["a"])];
    let before = row_motion_keys(&rows, None);
    let removed = row_motion_keys(&rows[1..], None);
    assert_eq!(removed[0], before[1]);
    assert_eq!(removed[1], before[0]); // Indistinguishable duplicate #0 remains.
    assert_ne!(before[0], before[2]);
    let sorted = row_motion_keys(&[rows[1].clone(), rows[0].clone(), rows[2].clone()], None);
    assert_eq!(
        sorted,
        vec![before[1].clone(), before[0].clone(), before[2].clone()]
    );
    let keyed = row_motion_keys(&[json!(["old", "id"])], Some(1));
    assert_eq!(keyed, row_motion_keys(&[json!(["new", "id"])], Some(1)));
}

#[test]
fn explicit_key_column_is_validated() {
    let widget = |column| {
        Widget::decode(&json!({"id":"list","kind":"list","label":"Rows","initial":0,"properties":{"dataset":"rows","row_count":2,"key_column":column}})).unwrap()
    };
    let model = json!({"datasets":{"rows":[["same","a"],["same","b"]]}});
    assert!(widget(json!(1)).validate_model(&model).is_ok());
    for column in [json!(-1), json!(0.5), json!("key"), json!(2), json!(0)] {
        assert!(widget(column).validate_model(&model).is_err());
    }
    assert!(
        widget(json!(1))
            .validate_model(&json!({"datasets":{"rows":[["a",""],["b","id"]]}}))
            .is_err()
    );
}
