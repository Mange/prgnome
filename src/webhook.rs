extern crate serde_json;

#[derive(Debug)]
pub enum Event {
    PullRequest(PullRequestEvent),
    Unknown {
        name: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PullRequestEvent {
    Labeled {
        label: Label,
        pull_request: PullRequest,
        repository: Repository,
        installation: Installation,
    },
    Unlabeled {
        label: Label,
        pull_request: PullRequest,
        repository: Repository,
        installation: Installation,
    },

    #[serde(other)]
    Other, // { payload: serde_json::Value, },
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub labels: Vec<Label>,
    pub head: Commit,
    pub base: Commit,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: u64,
}

#[derive(Debug, Fail)]
pub enum WebhookError {
    #[fail(display = "JSON parsing error")]
    ParseError(#[cause] serde_json::Error),
}

impl Event {
    pub fn parse_json(event_name: &str, json: &str) -> Result<Event, WebhookError> {
        match event_name {
            "pull_request" => PullRequestEvent::parse_json(json).map(Event::PullRequest),
            _ => serde_json::from_str(json)
                .map(|value| Event::Unknown {
                    name: event_name.to_owned(),
                    payload: value,
                }).map_err(WebhookError::from),
        }
    }
}

impl PullRequestEvent {
    pub fn parse_json(json: &str) -> Result<PullRequestEvent, WebhookError> {
        serde_json::from_str(json).map_err(WebhookError::from)
    }

    pub fn repo_url(&self) -> Option<&str> {
        match self {
            PullRequestEvent::Labeled { repository, .. } => Some(&repository.url),
            PullRequestEvent::Unlabeled { repository, .. } => Some(&repository.url),
            PullRequestEvent::Other => None,
        }
    }

    pub fn pull_request(&self) -> Option<&PullRequest> {
        match self {
            PullRequestEvent::Labeled { pull_request, .. } => Some(pull_request),
            PullRequestEvent::Unlabeled { pull_request, .. } => Some(pull_request),
            PullRequestEvent::Other => None,
        }
    }

    pub fn installation(&self) -> Option<&Installation> {
        match self {
            PullRequestEvent::Labeled { installation, .. } => Some(installation),
            PullRequestEvent::Unlabeled { installation, .. } => Some(installation),
            PullRequestEvent::Other => None,
        }
    }
}

impl From<serde_json::Error> for WebhookError {
    fn from(error: serde_json::Error) -> WebhookError {
        WebhookError::ParseError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_fixture(name: &str) -> String {
        let path = format!("tests/fixtures/{}", name);
        ::std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("Failed to read {}: {}", path, err);
        })
    }

    #[test]
    fn it_parses_labeled_pr_webhooks() {
        let data = read_fixture("webhook_pr_labeled.json");
        let event: Event = Event::parse_json("pull_request", &data).unwrap();
        match event {
            Event::PullRequest(PullRequestEvent::Labeled { .. }) => {}
            other => panic!(
                "Parsed as a {:#?}, but expected an Event::PullRequest(PullRequest::Labeled)",
                other
            ),
        }
    }

    #[test]
    fn it_parses_unlabeled_pr_webhooks() {
        let data = read_fixture("webhook_pr_unlabeled.json");
        let event: Event = Event::parse_json("pull_request", &data).unwrap();
        match event {
            Event::PullRequest(PullRequestEvent::Unlabeled { .. }) => {}
            other => panic!(
                "Parsed as a {:#?}, but expected an Event::PullRequest(PullRequest::Unlabeled)",
                other
            ),
        }
    }

    #[test]
    fn it_parses_other_pr_webhooks() {
        let data = read_fixture("webhook_pr_closed.json");
        let event: Event = Event::parse_json("pull_request", &data).unwrap();
        match event {
            Event::PullRequest(PullRequestEvent::Other { .. }) => {
                // assert_eq!(data, payload);
            }
            other => panic!(
                "Parsed as a {:#?}, but expected an Event::PullRequest(PullRequest::Other)",
                other
            ),
        }
    }

    #[test]
    fn it_stores_payload_on_unknown_events() {
        let data = r#"{"hello":"world"}"#;
        let data_value: serde_json::Value = serde_json::from_str(data).unwrap();

        let event: Event = Event::parse_json("foo", data).unwrap();
        match event {
            Event::Unknown { name, payload } => {
                assert_eq!(name, "foo");
                assert_eq!(data_value, payload);
            }
            other => panic!("Parsed as a {:#?}, but expected an Event::Unknown", other),
        }
    }
}
