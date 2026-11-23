// Merge-queue rehearsal (lab only, never lands on main): a seeded failure
// that only the full candidate gate runs, so the candidate must go red.
#[test]
fn lab_seeded_red() {
    assert_eq!(1 + 1, 3, "seeded red for the merge-queue rehearsal");
}
