//! Raw, read-only access to any endpoint, used by the recorder.

use reqwest::StatusCode;
use serde_json::Value;

use crate::{HttpLolClient, LolError};

/// Which local API an endpoint path belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Api {
    /// The League client (LCU), authenticated with the lockfile password.
    Lcu,
    /// The Live Client Data API, exposed by the game process.
    Live,
}

impl HttpLolClient {
    /// `GET`s `path` and returns its JSON body.
    ///
    /// `Ok(None)` means the endpoint is not available right now (404, or the
    /// API is not listening). This only ever issues `GET` requests: the
    /// recorder must observe, never change anything.
    pub async fn fetch_json(&self, api: Api, path: &str) -> Result<Option<Value>, LolError> {
        let (base, request) = match api {
            Api::Lcu => {
                let base = self.lockfile.base_url();
                let request = self
                    .http
                    .get(format!("{base}{path}"))
                    .basic_auth("riot", Some(&self.lockfile.password));
                (base, request)
            }
            Api::Live => {
                let base = self.live_client_url.clone();
                (base.clone(), self.http.get(format!("{base}{path}")))
            }
        };
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if error.is_connect() => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        match response.status() {
            status if status.is_success() => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            status => Err(LolError::UnexpectedStatus {
                status: status.as_u16(),
                url: format!("{base}{path}"),
            }),
        }
    }
}
