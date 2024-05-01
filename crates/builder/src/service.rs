use std::{fmt::Debug, net::SocketAddr};

use alloy_consensus::TxEnvelope;
use alloy_json_rpc::{ErrorPayload, Id};
use alloy_network::eip2718::Decodable2718;
use alloy_primitives::B256;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::{Instrument, Span};

/// App result
pub type AppResult<T, E = AppError> = Result<T, E>;

/// App error. This is a wrapper around eyre::Report that also includes an HTTP
/// status code. It implements [`IntoResponse`] so that it can be returned as an
/// error type from [`axum::handler::Handler`]s.
#[derive(Debug)]
pub struct AppError {
    code: StatusCode,
    eyre: eyre::Report,
}

impl AppError {
    /// Instantiate a new error with the bad request status code.
    pub fn bad_req<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            eyre: e.into(),
        }
    }

    /// Instantiate a new error with the bad request status code and an error
    /// string.
    pub fn bad_req_str(e: &str) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            eyre: eyre::eyre!(e.to_owned()),
        }
    }

    /// Instantiate a new error with the internal server error status code.
    pub fn server_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            eyre: e.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.code, format!("{}", self.eyre)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct ServiceState {
    dispatch: mpsc::UnboundedSender<TxEnvelope>,
}

/// Return a 404 Not Found response
pub async fn return_404() -> Response {
    (StatusCode::NOT_FOUND, "not found").into_response()
}

/// Return a 200 OK response
pub async fn return_200() -> Response {
    (StatusCode::OK, "ok").into_response()
}

/// Dispatches a transaction to the backend.
pub async fn include_tx(state: ServiceState, tx: TxEnvelope) -> Result<B256, AppError> {
    // Simple check to see if the transaction is signed correctly.
    if let Err(e) = tx.recover_signer() {
        return Err(AppError::bad_req(e));
    }

    let hash = *tx.tx_hash();
    // send it to the backend
    state.dispatch.send(tx).map_err(AppError::server_err)?;
    // return the hash
    Ok(hash)
}

/// Handler for the /sendTransaction endpoint
pub async fn ingest_handler(
    State(state): State<ServiceState>,
    Json(tx): Json<TxEnvelope>,
) -> Result<Response, AppError> {
    let hash = include_tx(state, tx).await?;
    Ok(hash.to_string().into_response())
}

/// Handler for the /sendRawTransaction endpoint
pub async fn ingest_raw_handler(
    State(state): State<ServiceState>,
    body: String,
) -> Result<Response, AppError> {
    let body = body.strip_prefix("0x").unwrap_or(&body);
    let buf = hex::decode(body).map_err(AppError::bad_req)?;

    let envelope = TxEnvelope::decode_2718(&mut buf.as_slice()).map_err(AppError::bad_req)?;

    ingest_handler(State(state), Json(envelope)).await
}

/// Handler for the /rpc endpoint.
/// Simulates the eth_sendRawTransaction JSON-RPC method
pub async fn ingest_rpc_handler(
    State(state): State<ServiceState>,
    body: String,
) -> Result<Response, AppError> {
    // parse JSON-RPC values from request
    let json = serde_json::from_str::<Value>(&body).map_err(AppError::bad_req)?;
    let method = json["method"].as_str().expect("method not found");
    let tx = json["params"][0].as_str().expect("params malformed");

    let id = match &json["id"] {
        Value::Number(n) => Id::Number(n.as_u64().unwrap_or_default()),
        Value::String(s) => Id::String(s.clone()),
        _ => Id::None,
    };

    // MUST be eth_sendRawTransaction method
    if method != "eth_sendRawTransaction" {
        return Ok(Json(alloy_json_rpc::Response {
            payload: alloy_json_rpc::ResponsePayload::<(), ()>::Failure(ErrorPayload {
                code: -6969,
                message: "Method not found".to_string(),
                data: None,
            }),
            id,
        })
        .into_response());
    }

    // parse TxEnvelope
    let body: &str = tx.strip_prefix("0x").unwrap_or(tx);
    let buf = hex::decode(body).map_err(AppError::bad_req)?;
    let tx = TxEnvelope::decode_2718(&mut buf.as_slice()).map_err(AppError::bad_req)?;

    let hash = include_tx(state, tx).await?;

    // return JSON-RPC response
    let resp = alloy_json_rpc::Response {
        payload: alloy_json_rpc::ResponsePayload::<_, ()>::Success(hash),
        id,
    };

    Ok(Json(resp).into_response())
}

/// Serve a builder service on the given socket address.
pub fn serve_builder_with_span(
    dispatch: mpsc::UnboundedSender<TxEnvelope>,
    socket: impl Into<SocketAddr>,
    span: Span,
) -> tokio::task::JoinHandle<()> {
    let state = ServiceState { dispatch };

    let router: Router<ServiceState> = Router::new()
        .route("/sendTransaction", post(ingest_handler))
        .route("/sendRawTransaction", post(ingest_raw_handler))
        .route("/rpc", post(ingest_rpc_handler))
        .route("/healthcheck", get(return_200))
        .fallback(return_404);
    let app = router.with_state(state);

    let addr = socket.into();
    tokio::spawn(
        async move {
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => {
                    if let Err(err) = axum::serve(listener, app).await {
                        tracing::error!(%err, "serve failed");
                    }
                }
                Err(err) => {
                    tracing::error!(%err, "failed to bind to the address");
                }
            };
        }
        .instrument(span),
    )
}
