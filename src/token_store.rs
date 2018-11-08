use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_TTL: u64 = 30 * 60; // 30 minutes

#[derive(Debug, Default)]
pub struct TokenStore {
    tokens: HashMap<u64, (Instant, String)>,
}

impl TokenStore {
    pub fn add_token<S: Into<String>>(&mut self, installation_id: u64, token: S) {
        self.tokens
            .insert(installation_id, (Instant::now(), token.into()));
    }

    pub fn get_token(&self, installation_id: u64) -> Option<String> {
        match self.tokens.get(&installation_id) {
            Some((time, token)) if time.elapsed() < Duration::from_secs(MAX_TTL) => {
                Some(token.clone())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stores_and_retrieves_tokens() {
        let mut store = TokenStore::default();
        store.add_token(12, "foobar");
        assert_eq!(store.get_token(12), Some(String::from("foobar")));
        assert_eq!(store.get_token(12), Some(String::from("foobar")));
        assert_eq!(store.get_token(55), None);
    }

    #[test]
    fn it_does_not_retrieve_old_tokens() {
        let mut store = TokenStore::default();
        store.tokens.insert(
            12,
            (
                Instant::now() - Duration::from_secs(MAX_TTL + 1),
                String::from("foobar"),
            ),
        );
        assert_eq!(store.get_token(12), None);
    }
}
