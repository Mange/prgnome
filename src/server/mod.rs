mod index;
mod webhook;

use actix_web::{http, HttpResponse, Result};
use event::EventError;
use github_api::{ApiError, Client as GithubClient};
use std::sync::RwLock;
use token_store::TokenStore;
use utils::log_error_trace;

mod prelude {
    pub use super::ServerState;
    pub use actix_web::{HttpRequest, Result, State};
    pub use std::sync::Arc;
}

pub use self::index::handle_index;
pub use self::webhook::handle_webhook;

pub struct ServerState {
    api_client: GithubClient,
    webhook_secret: String,
    /// Store auth tokens for different installations. Tokens expire once in a while, but can be
    /// regenerated using the private key stored in GithubClient.
    auth_tokens: RwLock<TokenStore>,
}

impl ServerState {
    pub fn new(api_client: GithubClient, webhook_secret: &str) -> Self {
        ServerState {
            api_client: api_client,
            webhook_secret: webhook_secret.to_owned(),
            auth_tokens: RwLock::new(TokenStore::default()),
        }
    }

    /// Return active auth token for an installation, generating a new one if no active token is
    /// stored.
    fn get_or_create_auth_token(&self, installation_id: u64) -> Result<String, ApiError> {
        match self.get_auth_token(installation_id) {
            Some(token) => {
                debug!("Auth token present in cache");
                Ok(token)
            }
            None => {
                debug!("Auth token not in cache. Generate a new one…");
                let token = self.api_client.generate_auth_token(installation_id)?;
                debug!("Waiting for write lock to auth token cache");
                let mut tokens = self.auth_tokens.write().unwrap();
                tokens.add_token(installation_id, token.clone());
                debug!("Auth token written to cache. Releasing write lock.");
                Ok(token)
            }
        }
    }

    /// Read auth token for an installation; return `None` if no active key exists.
    fn get_auth_token(&self, installation_id: u64) -> Option<String> {
        let tokens = self.auth_tokens.read().ok()?;
        tokens.get_token(installation_id)
    }

    fn webhook_secret(&self) -> &str {
        &self.webhook_secret
    }
}

impl actix_web::ResponseError for EventError {
    fn error_response(&self) -> HttpResponse {
        log_error_trace(self);
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
    }
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        log_error_trace(self);
        HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
