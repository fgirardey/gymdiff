//! HTTP implementation of [`LolClient`], for the real client and the simulator.

use std::time::Duration;

use gymdiff_core::GamePhase;
use reqwest::StatusCode;

use crate::live_client::GameStats;
use crate::{Lockfile, LolClient, LolError};

/// HTTP implementation of [`LolClient`].
#[derive(Debug, Clone)]
pub struct HttpLolClient {
    pub(crate) http: reqwest::Client,
    pub(crate) lockfile: Lockfile,
    pub(crate) live_client_url: String,
}

impl HttpLolClient {
    /// Builds a client from a lockfile. The Live Client Data URL comes from
    /// `GYMDIFF_LIVE_CLIENT_URL` or defaults to [`DEFAULT_LIVE_CLIENT_URL`].
    pub fn new(lockfile: Lockfile) -> Result<Self, LolError> {
        let live_client_url = std::env::var(LIVE_CLIENT_URL_ENV)
            .unwrap_or_else(|_| DEFAULT_LIVE_CLIENT_URL.to_owned());
        Self::with_live_client_url(lockfile, live_client_url)
    }

    pub fn with_live_client_url(
        lockfile: Lockfile,
        live_client_url: impl Into<String>,
    ) -> Result<Self, LolError> {
        // Both local APIs use a certificate signed by Riot's own root CA.
        // TODO(security): trust Riot's root certificate (riotgames.pem) instead
        // of accepting any certificate. Only 127.0.0.1 is ever contacted.
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(3))
            .build()?;
        Ok(Self {
            http,
            lockfile,
            live_client_url: live_client_url.into(),
        })
    }

    #[must_use]
    pub fn lockfile(&self) -> &Lockfile {
        &self.lockfile
    }
}

impl LolClient for HttpLolClient {
    async fn gameflow_phase(&self) -> Result<GamePhase, LolError> {
        let url = format!(
            "{}/lol-gameflow/v1/gameflow-phase",
            self.lockfile.base_url()
        );
        let response = self
            .http
            .get(&url)
            .basic_auth("riot", Some(&self.lockfile.password))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(LolError::UnexpectedStatus {
                status: response.status().as_u16(),
                url,
            });
        }
        Ok(response.json().await?)
    }

    async fn game_stats(&self) -> Result<Option<GameStats>, LolError> {
        let url = format!("{}/liveclientdata/gamestats", self.live_client_url);
        let response = match self.http.get(&url).send().await {
            Ok(response) => response,
            // The game process is not running: no game in progress.
            Err(error) if error.is_connect() => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        match response.status() {
            status if status.is_success() => Ok(Some(response.json().await?)),
            // The API answers 404 while the game is loading.
            StatusCode::NOT_FOUND => Ok(None),
            status => Err(LolError::UnexpectedStatus {
                status: status.as_u16(),
                url,
            }),
        }
    }
}

/// Default base URL of the Live Client Data API (exposed by the game process).
pub const DEFAULT_LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999";

/// Environment variable overriding the Live Client Data API base URL
/// (the simulator serves it over plain HTTP).
pub const LIVE_CLIENT_URL_ENV: &str = "GYMDIFF_LIVE_CLIENT_URL";
