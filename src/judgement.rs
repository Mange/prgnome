const FORBIDDEN_INTROS: [&'static str; 3] = ["wip", "fixup!", "squash!"];
const FORBIDDEN_MESSAGES: [&'static str; 1] = ["tmp"];

const FORBIDDEN_LABELS: [&'static str; 9] = [
    "work in progress",
    "work-in-progress",
    "wip",
    "in progress",
    "don't merge",
    "do not merge",
    "wait",
    "not ready",
    "blocked",
];

#[derive(Debug, Default)]
pub struct Intel<'a> {
    pub label_names: Vec<&'a str>,
    pub total_commits: u64,
    pub commit_messages: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Judgement {
    Approved,
    NotApproved(String),
}

fn truncate(s: &str) -> &str {
    if s.len() > 30 {
        &s[0..29]
    } else {
        s
    }
}

impl<'a> Intel<'a> {
    pub fn validate(&self) -> Judgement {
        for message in &self.commit_messages {
            // No need to have very long messages here as we're only looking at the start of the
            // message, or for very short whole messages.
            let normalized = truncate(message).to_ascii_lowercase();

            if FORBIDDEN_MESSAGES.contains(&&*normalized) {
                return Judgement::NotApproved(format!("Rebase away \"{}\"", normalized));
            }

            for forbidden_intro in FORBIDDEN_INTROS.iter() {
                if normalized.starts_with(forbidden_intro) {
                    return Judgement::NotApproved(format!("Rebase away \"{}\"", normalized));
                }
            }
        }

        for name in &self.label_names {
            let normalized = name.to_ascii_lowercase();
            if FORBIDDEN_LABELS.contains(&&*normalized) {
                return Judgement::NotApproved(format!("Remove the \"{}\" label", name));
            }
        }

        Judgement::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_a_valid_whitelist() {
        // Whitelist must be lowercase
        for label in FORBIDDEN_LABELS.iter() {
            assert_eq!(&label.to_ascii_lowercase(), label);
        }

        for intro in FORBIDDEN_INTROS.iter() {
            assert_eq!(&intro.to_ascii_lowercase(), intro);
        }
    }

    #[test]
    fn it_allows_empty_intel() {
        let intel = Intel::default();
        assert_eq!(intel.validate(), Judgement::Approved);
    }

    #[test]
    fn it_forbids_intel_with_forbidden_labels() {
        let intel = Intel {
            label_names: vec!["do NOT merge"],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved(String::from("Remove the \"do NOT merge\" label"))
        );
    }

    #[test]
    fn it_forbids_intel_with_forbidden_commit_intros() {
        let intel = Intel {
            commit_messages: vec![
                String::from("Initial commit"),
                String::from("fixup! Initial commit"),
            ],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved(String::from("Rebase away \"fixup! initial commit\""))
        );
    }

    #[test]
    fn it_forbids_intel_with_forbidden_commit_messages() {
        let intel = Intel {
            commit_messages: vec![String::from("Initial commit"), String::from("tmp")],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved(String::from("Rebase away \"tmp\""))
        );
    }

    #[test]
    fn it_is_okay_with_tricky_but_ok_messages() {
        let intel = Intel {
            commit_messages: vec![
                String::from("Clear out tmp"),
                String::from("Activate WIP gateway"),
            ],
            ..Default::default()
        };

        assert_eq!(intel.validate(), Judgement::Approved);
    }
}
