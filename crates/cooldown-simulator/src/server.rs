//! The fake LCU and Live Client Data HTTP servers.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

use crate::scenario::Scenario;

/// Serves one fake API on `127.0.0.1:<port>` until the process stops.
pub async fn serve(api: Api, port: u16, simulator: Arc<Simulator>) -> anyhow::Result<()> {
    let app = Router::new()
        .fallback(move |state, headers, uri| handle(api, state, headers, uri))
        .with_state(simulator);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    tracing::info!(
        api = match api {
            Api::Lcu => "LCU",
            Api::Live => "Live Client Data",
        },
        %addr,
        "listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

// axum fallback handlers must be async even without awaits.
#[allow(clippy::unused_async)]
async fn handle(
    api: Api,
    State(simulator): State<Arc<Simulator>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if matches!(api, Api::Lcu) && !is_authorized(&headers, &simulator.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let snapshot = simulator.scenario.snapshot_at(simulator.scenario_time());
    let endpoints = match api {
        Api::Lcu => &snapshot.lcu,
        Api::Live => &snapshot.live,
    };
    match endpoints.get(uri.path()) {
        Some(body) => (
            [(header::CONTENT_TYPE, "application/json")],
            body.to_string(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Checks HTTP Basic auth `riot:<password>`, like the real LCU API.
fn is_authorized(headers: &HeaderMap, password: &str) -> bool {
    let expected = format!(
        "Basic {}",
        crate::base64::encode(format!("riot:{password}").as_bytes())
    );
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected)
}

/// Shared state of the running simulator.
pub struct Simulator {
    pub scenario: Scenario,
    pub started: Instant,
    pub speed: f64,
    pub looping: bool,
    pub password: String,
}

impl Simulator {
    fn scenario_time(&self) -> f64 {
        let t = self.started.elapsed().as_secs_f64() * self.speed;
        let duration = self.scenario.duration();
        if self.looping && duration > 0.0 {
            // Keep the last step visible for a while before restarting.
            t % (duration + 10.0)
        } else {
            t
        }
    }
}

/// Which fake API a server answers for.
#[derive(Clone, Copy)]
pub enum Api {
    Lcu,
    Live,
}
