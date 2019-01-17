const FORBIDDEN_INTROS: [&'static str; 3] = ["wip", "fixup!", "squash!"];
const FORBIDDEN_MESSAGES: [&'static str; 1] = ["tmp"];

const MAGIC_IGNORE_LABEL: &str = "prgnome ignore";

const FORBIDDEN_LABELS: [&'static str; 8] = [
    "work in progress",
    "wip",
    "in progress",
    "dont merge",
    "do not merge",
    "wait",
    "not ready",
    "blocked",
];

const MAX_COMMITS: u64 = 50;

#[derive(Debug, Default)]
pub struct Intel<'a> {
    pub label_names: Vec<&'a str>,
    pub total_commits: u64,
    pub commit_messages: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Judgement {
    Approved,
    NotApproved {
        main_problem: String,
        total_violations: usize,
    },
    ForceApproved(String),
}

impl Judgement {
    fn add_problem(&mut self, message: String) {
        match self {
            Judgement::ForceApproved(_) => {}
            Judgement::Approved => {
                *self = Judgement::NotApproved {
                    main_problem: message,
                    total_violations: 1,
                };
            }
            Judgement::NotApproved {
                ref mut total_violations,
                ..
            } => {
                *total_violations += 1;
            }
        }
    }
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
        let mut judgement = Judgement::Approved;

        if self.total_commits > MAX_COMMITS {
            judgement.add_problem(format!(
                "Rebase until you have {} commits or fewer",
                MAX_COMMITS
            ));
        }

        for message in &self.commit_messages {
            // No need to have very long messages here as we're only looking at the start of the
            // message, or for very short whole messages.
            let normalized = truncate(message).to_ascii_lowercase();

            if FORBIDDEN_MESSAGES.contains(&&*normalized) {
                judgement.add_problem(format!("Rebase away \"{}\"", normalized));
            }

            for forbidden_intro in FORBIDDEN_INTROS.iter() {
                if normalized.starts_with(forbidden_intro) {
                    judgement.add_problem(format!("Rebase away \"{}\"", normalized));
                }
            }
        }

        for name in &self.label_names {
            let normalized = normalize_label(name);
            if FORBIDDEN_LABELS.contains(&&*normalized) {
                judgement.add_problem(format!("Remove the \"{}\" label", name));
            }

            if normalized == MAGIC_IGNORE_LABEL {
                return Judgement::ForceApproved(format!("Tagged with {}", name));
            }
        }

        judgement
    }
}

fn normalize_label(name: &str) -> String {
    name.chars()
        .flat_map(|c: char| match c {
            val if val.is_alphanumeric() => Some(val.to_ascii_lowercase()),
            ' ' | '-' | '/' | ':' => Some(' '),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_normalizes_labels() {
        for (input, output) in &[
            ("Hello world", "hello world"),
            ("size:large", "size large"),
            ("look/at/mEeE!?", "look at meee"),
        ] {
            assert_eq!(&normalize_label(input), output);
        }
    }

    #[test]
    fn it_has_a_valid_whitelist() {
        // Whitelist must be normalized already
        for label in FORBIDDEN_LABELS.iter() {
            assert_eq!(&normalize_label(label), label);
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
            label_names: vec!["do NOT merge!!!!!"],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved {
                main_problem: String::from("Remove the \"do NOT merge!!!!!\" label"),
                total_violations: 1,
            },
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
            Judgement::NotApproved {
                main_problem: String::from("Rebase away \"fixup! initial commit\""),
                total_violations: 1,
            }
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
            Judgement::NotApproved {
                main_problem: String::from("Rebase away \"tmp\""),
                total_violations: 1,
            }
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

    #[test]
    fn it_forbids_too_many_commits() {
        let intel = Intel {
            total_commits: MAX_COMMITS + 1,
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved {
                main_problem: format!("Rebase until you have {} commits or fewer", MAX_COMMITS),
                total_violations: 1,
            }
        );
    }

    #[test]
    fn it_records_multiple_violations() {
        let intel = Intel {
            commit_messages: vec![
                String::from("Initial commit"),
                String::from("fixup! Initial commit"),
            ],
            label_names: vec!["Work-in-progress"],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::NotApproved {
                main_problem: String::from("Rebase away \"fixup! initial commit\""),
                total_violations: 2,
            }
        );
    }

    #[test]
    fn it_approves_all_issues_if_tagged_with_magic_label() {
        let intel = Intel {
            commit_messages: vec![
                String::from("Initial commit"),
                String::from("fixup! Initial commit"),
            ],
            label_names: vec!["Work-in-progress", "prgnome-ignore"],
            ..Default::default()
        };

        assert_eq!(
            intel.validate(),
            Judgement::ForceApproved(String::from("Tagged with prgnome-ignore")),
        );
    }
}
