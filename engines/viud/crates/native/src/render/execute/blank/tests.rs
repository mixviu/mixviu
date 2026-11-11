// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use super::*;
use std::{cell::{Cell, RefCell}, collections::BTreeMap};

#[derive(Default)]
struct Fake {
    properties: BTreeMap<&'static str, u32>,
    values: RefCell<BTreeMap<u32, u64>>,
    commits: RefCell<Vec<(bool, Vec<(u32, u64)>)>>,
    blobs: RefCell<Vec<Vec<u8>>>,
    destroyed: RefCell<Vec<u64>>,
    refuse: Cell<Option<bool>>,
}
impl Fake {
    fn ctm() -> Self {
        Self { properties: [("CTM", 1), ("DEGAMMA_LUT", 2), ("GAMMA_LUT", 3)].into(), ..Default::default() }
    }
}
impl Surface for Fake {
    fn property(&self, name: &str) -> Option<u32> { self.properties.get(name).copied() }
    fn value(&self, h: u32) -> Option<u64> { self.values.borrow().get(&h).copied() }
    fn blob(&self, bytes: &mut [u8]) -> Result<u64, String> {
        self.blobs.borrow_mut().push(bytes.to_vec()); Ok(99)
    }
    fn commit(&self, props: &[(u32, u64)], test: bool) -> Result<(), String> {
        self.commits.borrow_mut().push((test, props.to_vec()));
        if self.refuse.get() == Some(test) { return Err("refused".into()); }
        if !test { self.values.borrow_mut().extend(props.iter().copied()); }
        Ok(())
    }
    fn destroy(&self, b: u64) { self.destroyed.borrow_mut().push(b); }
}

#[test]
fn ctm_is_nine_zero_coefficients_and_wins_over_gamma() {
    let f = Fake::ctm();
    assert_eq!(load(&f).unwrap(), (1, 99));
    assert_eq!(&*f.blobs.borrow(), &[vec![0; 72]]);
    assert_eq!(f.values.borrow().get(&1), Some(&99));
    assert_eq!(f.values.borrow().get(&2), Some(&0));
    assert_eq!(f.values.borrow().get(&3), Some(&0));
    assert_eq!(f.commits.borrow().iter().map(|c| c.0).collect::<Vec<_>>(), [true, false]);
}

#[test]
fn gamma_uses_advertised_lut_size() {
    let f = Fake {
        properties: [("GAMMA_LUT", 3), ("GAMMA_LUT_SIZE", 4)].into(),
        values: RefCell::new([(4, 256)].into()), ..Default::default()
    };
    assert_eq!(load(&f).unwrap(), (3, 99));
    assert_eq!(f.blobs.borrow()[0], vec![0; 256 * 8]);
}

#[test]
fn no_property_or_invalid_gamma_size_never_commits() {
    let mut f = Fake::default();
    assert!(load(&f).is_err());
    f.properties.extend([("GAMMA_LUT", 3), ("GAMMA_LUT_SIZE", 4)]);
    for size in [0, u64::MAX] {
        f.values.borrow_mut().insert(4, size);
        assert!(load(&f).is_err());
    }
    assert!(f.commits.borrow().is_empty());
}

#[test]
fn refused_test_never_changes_properties_and_destroys_blob() {
    let f = Fake::ctm();
    f.refuse.set(Some(true));
    assert!(load(&f).is_err());
    assert_eq!(f.commits.borrow().len(), 1);
    assert!(f.values.borrow().is_empty());
    assert_eq!(*f.destroyed.borrow(), [99]);
}

#[test]
fn refused_real_commit_destroys_blob_and_reports_failure() {
    let f = Fake::ctm();
    f.refuse.set(Some(false));
    assert!(load(&f).is_err());
    assert_eq!(f.commits.borrow().len(), 2);
    assert!(f.values.borrow().is_empty());
    assert_eq!(*f.destroyed.borrow(), [99]);
}

#[test]
fn pause_shutdown_clear_uses_the_owned_property_then_resume_reloads() {
    let f = Fake::ctm();
    let (h, blob) = load(&f).unwrap();
    // `Blank::clear` (pause, DPMS and shutdown) uses this path.
    unload(&f, h, blob).unwrap();
    assert_eq!(f.values.borrow().get(&h), Some(&0));
    assert_eq!(*f.destroyed.borrow(), [blob]);
    load(&f).unwrap();
    assert_eq!(f.values.borrow().get(&h), Some(&99));
    assert_eq!(f.commits.borrow().iter().map(|c| c.0).collect::<Vec<_>>(), [true, false, true, false, true, false]);
}

#[test]
fn modeset_reapplies_only_when_the_owned_blob_is_lost() {
    let f = Fake::ctm();
    let (h, blob) = load(&f).unwrap();
    restore(&f, h, blob).unwrap();
    assert_eq!(f.commits.borrow().len(), 2);
    f.values.borrow_mut().insert(h, 0); // modeset reset
    restore(&f, h, blob).unwrap();
    assert_eq!(f.values.borrow().get(&h), Some(&blob));
    assert_eq!(f.commits.borrow().len(), 4);
}

#[test]
fn connector_sdr_and_hdr_plan_retains_blank_and_clears_foreign_luts() {
    // The connector's SDR and HDR branches both consume this plan unchanged.
    let f = Fake::ctm();
    f.values.borrow_mut().extend([(1, 99), (2, 18), (3, 19)]);
    let changes = color_plan(&f, Some((1, 99)));
    assert_eq!(changes, [(2, 0), (3, 0)]);
    tested(&f, &changes).unwrap();
    assert_eq!(f.values.borrow().get(&1), Some(&99));
    // A lost blank after a modeset is explicitly re-applied in that same plan.
    f.values.borrow_mut().insert(1, 0);
    assert_eq!(color_plan(&f, Some((1, 99))), [(1, 99)]);
    // Startup, with no owner, clears a previous process's sticky blank.
    f.values.borrow_mut().insert(1, 99);
    assert_eq!(color_plan(&f, None), [(1, 0)]);
}

#[derive(Default)]
struct FakeDevice {
    values: RefCell<BTreeMap<(u32, u32), u64>>,
    tests: RefCell<Vec<bool>>,
    refuse: Cell<Option<bool>>,
}
impl AtomicDevice for FakeDevice {
    fn commit(&self, changes: &[Change], test: bool) -> Result<(), String> {
        self.tests.borrow_mut().push(test);
        if self.refuse.get() == Some(test) { return Err("one CRTC refused".into()); }
        if !test {
            let mut next = self.values.borrow().clone();
            for change in changes {
                for (h, v) in &change.props { next.insert((change.crtc.into(), *h), *v); }
            }
            *self.values.borrow_mut() = next;
        }
        Ok(())
    }
}

fn two_outputs(value: u64) -> Vec<Change> {
    [1, 2].into_iter().map(|raw| Change {
        crtc: std::num::NonZeroU32::new(raw).unwrap().into(),
        props: vec![(3, value)],
    }).collect()
}

#[test]
fn two_output_refusal_never_partially_blanks_or_unblanks() {
    for fail_test in [true, false] {
        let f = FakeDevice::default();
        f.refuse.set(Some(fail_test));
        assert!(transaction(&f, &two_outputs(99)).is_err());
        assert!(f.values.borrow().is_empty());
        assert_eq!(f.tests.borrow().as_slice(), if fail_test { &[true][..] } else { &[true, false][..] });
        f.tests.borrow_mut().clear();
        f.values.borrow_mut().extend([((1, 3), 99), ((2, 3), 99)]);
        assert!(transaction(&f, &two_outputs(0)).is_err());
        assert!(f.values.borrow().values().all(|v| *v == 99));
    }
}

#[test]
fn two_output_success_is_one_test_then_one_commit() {
    let f = FakeDevice::default();
    transaction(&f, &two_outputs(99)).unwrap();
    assert_eq!(*f.tests.borrow(), [true, false]);
    assert_eq!(f.values.borrow().len(), 2);
    assert!(f.values.borrow().values().all(|v| *v == 99));
    transaction(&f, &two_outputs(0)).unwrap();
    assert!(f.values.borrow().values().all(|v| *v == 0));
}
