extern crate jsonwebtoken as jwt;
extern crate reqwest;

use reqwest::RequestBuilder;

/// Expiry time for a JWT token in seconds. 10 minutes is the maximum allowed.
///
/// JWT tokens are used to create normal auth tokens (that expire every hour).
const EXPIRY_SECONDS: u64 = 60; // 1 minute

#[derive(Debug, Fail)]
pub enum ApiError {
    #[fail(display = "Failed to generate JWT")]
    JwtError(#[cause] jwt::errors::Error),

    #[fail(display = "Network/API error")]
    NetworkError(#[cause] reqwest::Error),
}

pub struct Client {
    app_id: u64,
    private_key: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct Claims {
    iat: u64, // Issued At time
    exp: u64, // Expiry time (max 10 minutes)
    iss: u64, // Issuer
}

fn unix_timestamp() -> u64 {
    use std::time;
    let now = time::SystemTime::now();

    now.duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Client {
    pub fn new(app_id: u64, private_key: Vec<u8>) -> Client {
        Client {
            app_id,
            private_key,
        }
    }

    /// Given an installation ID, use the private key to generate a new access token for use with
    /// the other APIs.
    pub fn generate_auth_token(&self, installation_id: u64) -> Result<String, ApiError> {
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );
        let jwt = self.new_jwt()?;
        let mut response = reqwest::Client::new()
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github.machine-man-preview+json")
            .send()?
            .error_for_status()?;

        let body: InstallationAccessTokens = response.json()?;
        Ok(body.token)
    }

    pub fn create_status(
        &self,
        auth_token: &str,
        repo_url: &str,
        sha: &str,
        input: NewStatus,
    ) -> Result<Status, ApiError> {
        let full_path = format!("{repo}/statuses/{sha}", repo = repo_url, sha = sha);
        let client = reqwest::Client::new();

        let mut response = add_headers(client.post(&full_path), auth_token)
            .json(&input)
            .send()?
            .error_for_status()?;

        response.json().map_err(ApiError::from)
    }

    pub fn list_commits_in_range(
        &self,
        auth_token: &str,
        repo_url: &str,
        base_sha: &str,
        head_sha: &str,
    ) -> Result<CommitList, ApiError> {
        let full_path = format!(
            "{repo}/compare/{base}...{head}",
            repo = repo_url,
            base = base_sha,
            head = head_sha
        );
        let client = reqwest::Client::new();

        let mut response = add_headers(client.get(&full_path), auth_token)
            .send()?
            .error_for_status()?;

        response.json().map_err(ApiError::from)
    }

    fn new_jwt(&self) -> Result<String, ApiError> {
        use self::jwt::{Algorithm, Header};
        let now = unix_timestamp();

        let claims = Claims {
            iat: now,
            exp: now + EXPIRY_SECONDS,
            iss: self.app_id,
        };

        let mut header = Header::default();
        header.alg = Algorithm::RS256;

        jwt::encode(&header, &claims, &self.private_key).map_err(ApiError::from)
    }
}

fn add_headers(req: RequestBuilder, auth_token: &str) -> RequestBuilder {
    req.header("Accept", "application/vnd.github.machine-man-preview+json")
        .header("Authorization", format!("token {}", auth_token))
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> ApiError {
        ApiError::NetworkError(error)
    }
}

impl From<jwt::errors::Error> for ApiError {
    fn from(error: jwt::errors::Error) -> ApiError {
        ApiError::JwtError(error)
    }
}

#[derive(Debug, Serialize)]
pub struct NewStatus {
    pub state: State,
    pub context: String,

    pub target_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Error,
    Failure,
    Pending,
    Success,

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct CommitList {
    pub total_commits: u64,
    pub commits: Vec<CommitInfo>,
}

#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub commit: Commit,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct InstallationAccessTokens {
    token: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub state: State,
    pub target_url: Option<String>,
    pub description: Option<String>,
    pub context: Option<String>,
}
