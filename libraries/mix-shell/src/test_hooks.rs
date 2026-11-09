//! Fault injection is opt-in at build time, never a runtime production switch.
//!
//! A binary that links the hooks also carries `MARKER`, so the artefact itself
//! can be checked (`mix tools/ci.mix release-mix` refuses a release `mix`
//! that contains it). The build-script and `compile_error!` guards stop the
//! feature at build time; the marker catches any route around them.
#[cfg(feature = "test-hooks")]
static MARKER: &[u8] = b"mixviu-test-hooks-linked";

pub(crate) fn var(name: &str) -> Option<std::ffi::OsString> {
    #[cfg(feature = "test-hooks")]
    {
        // Kept live by every hook lookup, so the linker cannot drop it while
        // the hooks themselves are linked.
        std::hint::black_box(MARKER);
        std::env::var_os(name)
    }
    #[cfg(not(feature = "test-hooks"))]
    {
        let _ = name;
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hooks_follow_the_build_feature() {
        // Re-exec so the environment is set before any test threads start.
        const HOOK: &str = "MIX_TEST_RESULT_TORN";
        if std::env::var_os(HOOK).is_some() {
            for name in [
                HOOK,
                "MIX_TEST_ADMIT_DELAY_MS",
                "MIX_TEST_CLAIM_DELAY_MS",
                "MIX_TEST_RESERVE_HOLD_MS",
            ] {
                assert_eq!(super::var(name).is_some(), cfg!(feature = "test-hooks"));
            }
            return;
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "test_hooks::tests::hooks_follow_the_build_feature",
            ])
            .env(HOOK, "1")
            .env("MIX_TEST_ADMIT_DELAY_MS", "1")
            .env("MIX_TEST_CLAIM_DELAY_MS", "1")
            .env("MIX_TEST_RESERVE_HOLD_MS", "1")
            .status()
            .unwrap();
        assert!(status.success());
    }
}
