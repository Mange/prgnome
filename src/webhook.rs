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
    Labeled {},
    Unlabeled {},

    #[serde(other)]
    Other, // { payload: serde_json::Value, },
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
