// SPDX-License-Identifier: MIT OR Apache-2.0
//! Initial registration ladder; a reconnect never chooses another identity.
use bus::{CommandDescriptor, SupervisedClient};
use std::{future::Future, time::Duration};

pub(crate) struct Failure {
    pub refused: bool,
    pub message: String,
}

pub(crate) fn rename(command: &str, from: &str, to: &str) -> String {
    command
        .strip_prefix(from)
        .and_then(|s| s.strip_prefix('.'))
        .map(|s| format!("{to}.{s}"))
        .unwrap_or_else(|| command.to_owned())
}

async fn ladder<T, F, Fut>(
    base: &str,
    pid: u32,
    enabled: bool,
    mut connect: F,
) -> Result<(String, T), String>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = Result<T, Failure>>,
{
    match connect(base.to_owned()).await {
        Ok(client) => Ok((base.to_owned(), client)),
        Err(error) if enabled && error.refused => {
            let name = format!("{base}-{pid}");
            connect(name.clone())
                .await
                .map(|client| (name, client))
                .map_err(|e| e.message)
        }
        Err(error) => Err(error.message),
    }
}

pub(crate) async fn connect(
    base: &str,
    url: &str,
    commands: &[CommandDescriptor],
    with_commands: bool,
    enabled: bool,
) -> Result<(String, SupervisedClient), String> {
    ladder(base, std::process::id(), enabled, |name| {
        let mut described = commands.to_vec();
        for command in &mut described {
            command.name = rename(&command.name, base, &name);
        }
        async move {
            let options = SupervisedClient::connect_options(&name, url)
                .bounded_incoming_rpc(32)
                .fatal_on_registration_rejection(true);
            let options = if with_commands {
                options.with_commands(described)
            } else {
                options
            };
            let attempt = options.connect();
            let answer = if enabled {
                match tokio::time::timeout(Duration::from_secs(2), attempt).await {
                    Ok(answer) => answer,
                    Err(_) => {
                        return Err(Failure {
                            refused: false,
                            message: format!("{name} Bus connection timed out"),
                        });
                    }
                }
            } else {
                attempt.await
            };
            answer.map_err(|error| Failure {
                refused: error.registration_rejection().is_some(),
                message: format!("{name} Bus unavailable: {error}"),
            })
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(
        answers: Vec<Result<u8, Failure>>,
        enabled: bool,
    ) -> (Result<(String, u8), String>, Vec<String>) {
        let mut answers = answers.into_iter();
        let mut names = Vec::new();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = runtime.block_on(ladder("sample", 42, enabled, |name| {
            names.push(name);
            std::future::ready(answers.next().expect("unexpected third attempt"))
        }));
        (result, names)
    }
    fn failure(refused: bool, message: &str) -> Result<u8, Failure> {
        Err(Failure {
            refused,
            message: message.into(),
        })
    }
    #[test]
    fn registration_ladder_retains_base_and_retries_only_one_refusal() {
        assert_eq!(
            run(vec![Ok(7)], true),
            (Ok(("sample".into(), 7)), vec!["sample".into()])
        );
        assert_eq!(
            run(vec![failure(true, "held"), Ok(8)], true),
            (
                Ok(("sample-42".into(), 8)),
                vec!["sample".into(), "sample-42".into()]
            )
        );
        assert_eq!(
            run(
                vec![
                    failure(true, "held"),
                    failure(true, "policy refused suffix")
                ],
                true
            ),
            (
                Err("policy refused suffix".into()),
                vec!["sample".into(), "sample-42".into()]
            )
        );
    }
    #[test]
    fn transport_timeout_and_disabled_policy_never_rename() {
        for message in [
            "transport failed",
            "connection timed out",
            "authentication failed",
        ] {
            assert_eq!(
                run(vec![failure(false, message)], true),
                (Err(message.into()), vec!["sample".into()])
            );
        }
        assert_eq!(
            run(vec![failure(true, "held")], false),
            (Err("held".into()), vec!["sample".into()])
        );
    }
    #[test]
    fn namespace_mapping_requires_a_dot_boundary_and_preserves_other_commands() {
        assert_eq!(
            rename("sample.ui.capture", "sample", "sample-42"),
            "sample-42.ui.capture"
        );
        assert_eq!(
            rename("sample-42.ui.capture", "sample-42", "sample"),
            "sample.ui.capture"
        );
        for command in ["sample", "samples.show", "HELP", "app.describe", "viu.quit"] {
            assert_eq!(rename(command, "sample", "sample-42"), command);
        }
    }
}
