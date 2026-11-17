// SPDX-License-Identifier: MIT OR Apache-2.0
//! Linux pdeath_signal belongs to a task. Transfer the initial task's
//! inherited setting to the evaluation task before that task execs mixd.
use std::{env, io, sync::OnceLock};

struct Inherited {
    parent: libc::pid_t,
    requested: bool,
    signal: Result<libc::c_int, String>,
}

static INHERITED: OnceLock<Inherited> = OnceLock::new();

fn check_parent(expected: libc::pid_t, actual: libc::pid_t) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "die_with_parent: expected parent {expected}, found {actual}; parent died before startup or engine handoff"
        ))
    }
}

pub(super) fn capture() -> Result<(), String> {
    let marker = env::var_os(mix::builtins::EXPECTED_PARENT_ENV);
    // SAFETY: called on the initial thread before native_session or the
    // evaluator starts. This one-hop marker must not reach unrelated children.
    unsafe { env::remove_var(mix::builtins::EXPECTED_PARENT_ENV) };
    let actual = unsafe { libc::getppid() };
    let mut signal = 0;
    // SAFETY: PR_GET_PDEATHSIG writes an int to the live local.
    let signal = if unsafe { libc::prctl(libc::PR_GET_PDEATHSIG, &mut signal) } == -1 {
        Err(io::Error::last_os_error().to_string())
    } else {
        Ok(signal)
    };
    let parent = scoped_parent(marker.as_deref(), &signal, actual)?;
    // A marker is ambient data until the kernel confirms this task is armed.
    // fork clears PDEATHSIG, so intermediaries' grandchildren ignore it too.
    let _ = INHERITED.set(Inherited {
        parent,
        requested: matches!(signal, Ok(value) if value != 0),
        signal,
    });
    Ok(())
}

fn scoped_parent(
    marker: Option<&std::ffi::OsStr>,
    signal: &Result<libc::c_int, String>,
    actual: libc::pid_t,
) -> Result<libc::pid_t, String> {
    if !matches!(signal, Ok(value) if *value != 0) {
        return Ok(actual);
    }
    let parent = match marker {
        Some(value) => value.to_str().and_then(|s| s.parse::<libc::pid_t>().ok())
            .filter(|pid| *pid > 0)
            .ok_or_else(|| "die_with_parent: invalid expected parent PID".to_string())?,
        None => actual,
    };
    check_parent(parent, actual)?;
    Ok(parent)
}

fn restore(
    state: &Inherited,
    mut parent: impl FnMut() -> libc::pid_t,
    arm: impl FnOnce(libc::c_int) -> Result<(), String>,
) -> Result<Option<String>, String> {
    let signal = match &state.signal {
        Ok(0) => return Ok(None),
        Ok(signal) => *signal,
        Err(error) => {
            if !state.requested {
                return Ok(None);
            }
            check_parent(state.parent, parent())?;
            return Ok(Some(format!(
                "cannot read parent-death signal: {error}; continuing without engine handoff lifetime tie"
            )));
        }
    };
    check_parent(state.parent, parent())?;
    let warning = arm(signal).err().map(|error| format!("cannot restore parent-death signal: {error}; continuing without engine handoff lifetime tie"));
    // Arm before checking again, closing parent death during the transfer.
    check_parent(state.parent, parent())?;
    Ok(warning)
}

pub(super) fn handoff() -> Result<(), String> {
    if let Some(state) = INHERITED.get() {
        let warning = restore(
            state,
            || unsafe { libc::getppid() },
            |signal| {
                if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, signal, 0, 0, 0) } == -1 {
                    Err(io::Error::last_os_error().to_string())
                } else {
                    Ok(())
                }
            },
        )?;
        if let Some(warning) = warning {
            eprintln!("mix: warning: {warning}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambient_marker_is_inert_unless_kernel_confirms_armed_task() {
        for marker in ["1", "not-a-pid"] {
            assert_eq!(scoped_parent(Some(marker.as_ref()), &Ok(0), 42).unwrap(), 42);
            assert_eq!(scoped_parent(Some(marker.as_ref()), &Err("injected EPERM".into()), 42).unwrap(), 42);
            assert!(scoped_parent(Some(marker.as_ref()), &Ok(libc::SIGKILL), 42).is_err());
        }
    }

    #[test]
    fn refused_read_is_silent_without_a_request_and_warns_only_at_handoff() {
        for requested in [false, true] {
            let state = Inherited {
                parent: 42,
                requested,
                signal: Err("injected EPERM".into()),
            };
            let warning =
                restore(&state, || 42, |_| panic!("must not arm after refused read")).unwrap();
            assert_eq!(warning.is_some(), requested);
            if let Some(warning) = warning {
                assert!(warning.contains("continuing without"));
            }
        }
    }

    #[test]
    fn refused_restore_warns_and_continues() {
        let state = Inherited {
            parent: 42,
            requested: true,
            signal: Ok(libc::SIGKILL),
        };
        let warning = restore(&state, || 42, |_| Err("injected EPERM".into()))
            .unwrap()
            .unwrap();
        assert!(warning.contains("cannot restore"));
    }

    #[test]
    fn spawn_parent_baseline_detects_death_before_main_and_during_handoff() {
        assert!(
            check_parent(42, 1)
                .unwrap_err()
                .contains("expected parent 42, found 1")
        );
        let state = Inherited {
            parent: 42,
            requested: true,
            signal: Ok(libc::SIGKILL),
        };
        let mut calls = 0;
        let error = restore(
            &state,
            || {
                calls += 1;
                if calls == 1 { 42 } else { 1 }
            },
            |_| Ok(()),
        )
        .unwrap_err();
        assert!(error.contains("parent died"));
    }
}
