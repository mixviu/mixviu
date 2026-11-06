// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
#[test]
fn timers_are_once_cancelable_and_refuse_invalid_replacements_atomically() {
    let timers = viu::timers::Timers::default();
    timers
        .arm("now", Some(0.0), json!({"kind":"echo"}))
        .unwrap();
    assert_eq!(
        timers.poll().0,
        vec![json!({"timer":"now","value":{"kind":"echo"}})]
    );
    assert!(timers.poll().0.is_empty());
    assert!(timers.poll().1.is_none());
    timers.arm("later", Some(5000.0), json!(1)).unwrap();
    assert!(timers.arm("later", Some(f64::NAN), json!(2)).is_err());
    assert!(timers.poll().1.is_some());
    timers.arm("later", None, json!(null)).unwrap();
    assert!(timers.poll().1.is_none());
    for i in 0..128 {
        timers.arm(&i.to_string(), Some(1000.0), json!(i)).unwrap();
    }
    assert!(timers.arm("overflow", Some(1.0), json!(0)).is_err());
    timers.arm("0", Some(0.0), json!("replacement")).unwrap();
    assert_eq!(timers.poll().0.len(), 1);
}
