use super::prelude::*;
use crate::webhook::Event;
use github_api::{NewStatus, State as CommitState};
use judgement::*;
use utils::log_error_trace_if_err;

const STATUS_CONTEXT_NAME: &str = "mange/prgnome";

pub fn handle_webhook(
    state: State<Arc<ServerState>>,
    event_name: EventName,
    body: String,
) -> Result<String> {
    let event = Event::parse_json(&event_name.0, &body)?;
    debug!("{:#?}", event);

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

                let intel = Intel { label_names };

                let judgement = intel.validate();
                let new_status = new_status_from_judgment(&judgement);

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

fn new_status_from_judgment(judgement: &Judgement) -> NewStatus {
    let (state, description) = match judgement {
        Judgement::Approved => (CommitState::Success, None),
        Judgement::NotApproved => (
            CommitState::Failure,
            Some("Failure description is not yet implemented"),
        ),
    };

    NewStatus {
        state: state,
        description: description.map(String::from),
        context: STATUS_CONTEXT_NAME.into(),
        target_url: None,
    }
}

#[derive(Debug)]
pub struct EventName(pub String);

impl<S> actix_web::FromRequest<S> for EventName {
    type Config = ();
    type Result = Result<EventName>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        debug!("headers: {:#?}", req.headers());
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
