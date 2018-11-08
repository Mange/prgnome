const FORBIDDEN_INTROS: [&'static str; 4] = ["wip", "fixup!", "squash!", "tmp"];

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
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Judgement {
    Approved,
    NotApproved,
}

impl<'a> Intel<'a> {
    pub fn validate(&self) -> Judgement {
        for name in &self.label_names {
            let normalized = name.to_ascii_lowercase();
            if FORBIDDEN_LABELS.contains(&&*normalized) {
                return Judgement::NotApproved;
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

        assert_eq!(intel.validate(), Judgement::NotApproved);
    }
}
