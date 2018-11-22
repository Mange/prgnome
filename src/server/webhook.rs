use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha1::Sha1;
use hex::FromHex;

use super::prelude::*;
use crate::webhook::Event;
use github_api::{Client as GithubClient, NewStatus, State as CommitState};
use judgement::*;
use utils::{log_error_trace, log_error_trace_if_err};

const STATUS_CONTEXT_NAME: &str = "mange/prgnome";

pub fn handle_webhook(
    state: State<Arc<ServerState>>,
    event_name: EventName,
    signature: GithubSignature,
    body: String,
) -> Result<String> {
    if !verify_signature(&body, &signature, state.webhook_secret()) {
        warn!("Webhook signature verification failed.");
        return Err(actix_web::Error::from(format_err!(
            "Signature could not be verified",
        )));
    }

    let event = Event::parse_json(&event_name.0, &body)?;
    match event {
        Event::PullRequest(pr_event) => {
            if let (Some(repo_url), Some(pr), Some(installation)) = (
                pr_event.repo_url(),
                pr_event.pull_request(),
                pr_event.installation(),
            ) {
                let auth_token = state.get_or_create_auth_token(installation.id)?;
                let label_names: Vec<&str> =
                    pr.labels.iter().map(|label| label.name.as_str()).collect();
                let (total_commits, commit_messages) = load_commits(
                    &state.api_client,
                    repo_url,
                    &auth_token,
                    &pr.base.sha,
                    &pr.head.sha,
                ).unwrap_or_else(|err| {
                    log_error_trace(err.as_fail());
                    Default::default()
                });

                let intel = Intel {
                    label_names,
                    total_commits,
                    commit_messages,
                };

                let judgement = intel.validate();
                let new_status = new_status_from_judgement(judgement);
                info!("Setting new status to: {:#?}", new_status);

                log_error_trace_if_err(&state.api_client.create_status(
                    &auth_token,
                    repo_url,
                    &pr.head.sha,
                    new_status,
                ));
            }
        }
        _ => {}
    }

    Ok(format!("OK"))
}

fn load_commits(
    api_client: &GithubClient,
    repo_url: &str,
    auth_token: &str,
    base_sha: &str,
    head_sha: &str,
) -> Result<(u64, Vec<String>)> {
    let commit_list = api_client.list_commits_in_range(auth_token, repo_url, base_sha, head_sha)?;
    Ok((
        commit_list.total_commits,
        commit_list
            .commits
            .into_iter()
            .map(|c| c.commit.message)
            .collect(),
    ))
}

fn new_status_from_judgement(judgement: Judgement) -> NewStatus {
    let (state, description) = match judgement {
        Judgement::Approved => (CommitState::Success, None),
        Judgement::ForceApproved(reason) => (CommitState::Success, Some(reason)),
        Judgement::NotApproved {
            main_problem,
            total_violations,
        } => {
            let message = if total_violations == 1 {
                main_problem
            } else {
                format!("{} problems. First one: {}", total_violations, main_problem)
            };
            (CommitState::Failure, Some(message))
        }
    };

    NewStatus {
        state: state,
        description,
        context: STATUS_CONTEXT_NAME.into(),
        target_url: None,
    }
}

fn verify_signature(payload: &str, signature: &str, secret: &str) -> bool {
    // https://developer.github.com/webhooks/securing/#validating-payloads-from-github
    let signature = &signature[5..signature.len()]; // cut off "sha1="
    debug!("Verifying webhook signature");

    let signature_bytes = match Vec::from_hex(signature) {
        Ok(val) => val,
        Err(err) => {
            error!(
                "Failed to parse {} as hex-encoded bytes: {}",
                signature, err
            );
            return false;
        }
    };

    let mut hmac = Hmac::new(Sha1::new(), secret.as_bytes());

    hmac.input(payload.as_bytes());

    // Secure compare helper in MacResult
    hmac.result() == MacResult::new(&signature_bytes)
}

#[derive(Debug)]
pub struct EventName(pub String);

#[derive(Debug)]
pub struct GithubSignature(pub String);

impl<S> actix_web::FromRequest<S> for EventName {
    type Config = ();
    type Result = Result<EventName>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        let header = req
            .headers()
            .get("X-Github-Event")
            .ok_or_else(|| format_err!("No X-Github-Event header present"))?;

        header
            .to_str()
            .map(String::from)
            .map(EventName)
            .map_err(|err| {
                actix_web::Error::from(format_err!("Cannot parse X-Github-Event header: {}", err))
            })
    }
}

impl<S> actix_web::FromRequest<S> for GithubSignature {
    type Config = ();
    type Result = Result<GithubSignature>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        let header = req
            .headers()
            .get("X-Hub-Signature")
            .ok_or_else(|| format_err!("No X-Hub-Signature header present"))?;

        header
            .to_str()
            .map(String::from)
            .map(GithubSignature)
            .map_err(|err| {
                actix_web::Error::from(format_err!("Cannot parse X-Hub-Signature header: {}", err))
            })
    }
}

impl std::ops::Deref for GithubSignature {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new_status_from_judgement {
        use super::*;

        #[test]
        fn it_returns_success_on_approved_judgement() {
            let judgement = Judgement::Approved;
            let new_state = new_status_from_judgement(judgement);

            assert_eq!(new_state.state, CommitState::Success);
            assert_eq!(new_state.description, None);
        }

        #[test]
        fn it_returns_failure_on_not_approved_judgement_with_single_problem() {
            let judgement = Judgement::NotApproved {
                main_problem: String::from("Not cool enough"),
                total_violations: 1,
            };
            let new_state = new_status_from_judgement(judgement);

            assert_eq!(new_state.state, CommitState::Failure);
            assert_eq!(new_state.description, Some(String::from("Not cool enough")));
        }

        #[test]
        fn it_returns_failure_on_not_approved_judgement_with_multiple_problems() {
            let judgement = Judgement::NotApproved {
                main_problem: String::from("Not cool enough"),
                total_violations: 4,
            };
            let new_state = new_status_from_judgement(judgement);

            assert_eq!(new_state.state, CommitState::Failure);
            assert_eq!(
                new_state.description,
                Some(String::from("4 problems. First one: Not cool enough")),
            );
        }

        #[test]
        fn it_returns_success_on_force_approved_judgement() {
            let judgement = Judgement::ForceApproved(String::from("Tagged with something cool"));
            let new_state = new_status_from_judgement(judgement);

            assert_eq!(new_state.state, CommitState::Success);
            assert_eq!(
                new_state.description,
                Some(String::from("Tagged with something cool")),
            );
        }
    }
}
