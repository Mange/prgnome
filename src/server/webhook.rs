use super::prelude::*;
use crate::webhook::Event;

pub fn handle_webhook(
    state: State<Arc<ServerState>>,
    event_name: EventName,
    body: String,
) -> Result<String> {
    let event = Event::parse_json(&event_name.0, &body)?;
    println!("{:#?}", event);

    match event {
        Event::PullRequest(pr_event) => {
            //
            // As an experiment, load information about the PR. This is out bootstrapping example.
            //
            if let (Some(repo_url), Some(pr), Some(installation)) = (
                pr_event.repo_url(),
                pr_event.pull_request(),
                pr_event.installation(),
            ) {
                let auth_token = state.get_or_create_auth_token(installation.id)?;
                println!(
                    "{:#?}",
                    state
                        .api_client
                        .statuses(&auth_token, repo_url, &pr.head.sha)
                );
            }
        }
        _ => {}
    }

    Ok(format!("OK"))
}

#[derive(Debug)]
pub struct EventName(pub String);

impl<S> actix_web::FromRequest<S> for EventName {
    type Config = ();
    type Result = Result<EventName>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        let header = req
            .headers()
            .get("Github-Event")
            .ok_or_else(|| format_err!("No Github-Event header present"))?;

        header
            .to_str()
            .map(String::from)
            .map(EventName)
            .map_err(|err| {
                actix_web::Error::from(format_err!("Cannot parse Github-Event header: {}", err))
            })
    }
}

