//! HTTP endpoints implemented by the x402 **facilitator**.
//!
//! These are the server-side handlers for processing client-submitted x402 payments.
//! They include both protocol-critical endpoints (`/verify`, `/settle`) and discovery endpoints (`/supported`, etc).
//!
//! All payloads follow the types defined in the `x402-rs` crate, and are compatible
//! with the TypeScript and Go client SDKs.
//!
//! Each endpoint consumes or produces structured JSON payloads defined in `x402-rs`,
//! and is compatible with official x402 client SDKs.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router, response::IntoResponse};
use serde_json::json;
use tracing::instrument;

use crate::chain::FacilitatorLocalError;
use crate::facilitator::Facilitator;
use crate::types::{
    ErrorResponse, FacilitatorErrorReason, MixedAddress, SettleRequest, VerifyRequest,
    VerifyResponse,
};

/// `GET /verify`: Returns a machine-readable description of the `/verify` endpoint.
///
/// This is served by the facilitator to help clients understand how to construct
/// a valid [`VerifyRequest`] for payment verification.
///
/// This is optional metadata and primarily useful for discoverability and debugging tools.
#[instrument(skip_all)]
pub async fn get_verify_info() -> impl IntoResponse {
    Json(json!({
        "endpoint": "/verify",
        "description": "POST to verify x402 payments",
        "body": {
            "paymentPayload": "PaymentPayload",
            "paymentRequirements": "PaymentRequirements",
        }
    }))
}

/// `GET /settle`: Returns a machine-readable description of the `/settle` endpoint.
///
/// This is served by the facilitator to describe the structure of a valid
/// [`SettleRequest`] used to initiate on-chain payment settlement.
#[instrument(skip_all)]
pub async fn get_settle_info() -> impl IntoResponse {
    Json(json!({
        "endpoint": "/settle",
        "description": "POST to settle x402 payments",
        "body": {
            "paymentPayload": "PaymentPayload",
            "paymentRequirements": "PaymentRequirements",
        }
    }))
}

pub fn routes<A>() -> Router<A>
where
    A: Facilitator + Clone + Send + Sync + 'static,
    A::Error: IntoResponse,
{
    use tower_http::services::ServeDir;

    Router::new()
        .route("/", get(get_root))
        .route("/verify", get(get_verify_info))
        .route("/verify", post(post_verify::<A>))
        .route("/settle", get(get_settle_info))
        .route("/settle", post(post_settle::<A>))
        .route("/health", get(get_health::<A>))
        .route("/supported", get(get_supported::<A>))
        .nest_service("/static", ServeDir::new("static"))
}

/// `GET /`: Returns the Stake Capital branded landing page.
#[instrument(skip_all)]
pub async fn get_root() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>STAKE CAPITAL | x402 Facilitator</title>
    <link rel="icon" href="/static/favicon.ico" type="image/x-icon">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            background: #ffffff;
            font-family: 'Courier New', Courier, monospace;
            color: #000000;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 2rem;
        }

        .container {
            max-width: 800px;
            text-align: center;
        }

        .logo {
            width: 256px;
            height: 256px;
            margin: 0 auto 3rem;
        }

        h1 {
            font-size: 2.5rem;
            letter-spacing: 0.1em;
            margin-bottom: 1.5rem;
            font-weight: 700;
        }

        .tagline {
            font-size: 1.1rem;
            margin-bottom: 3rem;
            line-height: 1.6;
            opacity: 0.8;
        }

        .endpoints {
            background: #f5f5f5;
            border: 1px solid #000000;
            padding: 2rem;
            margin: 2rem 0;
            text-align: left;
        }

        .endpoints h2 {
            font-size: 1.5rem;
            margin-bottom: 1.5rem;
            letter-spacing: 0.05em;
        }

        .endpoint-list {
            list-style: none;
        }

        .endpoint-list li {
            margin-bottom: 1rem;
            font-size: 1rem;
            line-height: 1.8;
        }

        .endpoint-list code {
            background: #ffffff;
            padding: 0.3rem 0.6rem;
            border: 1px solid #000000;
            font-family: 'Courier New', Courier, monospace;
            font-size: 0.95rem;
        }

        .endpoint-list .method {
            font-weight: bold;
            display: inline-block;
            width: 60px;
        }

        .footer {
            margin-top: 3rem;
            font-size: 0.9rem;
            opacity: 0.6;
        }

        a {
            color: #000000;
            text-decoration: underline;
        }

        a:hover {
            text-decoration: none;
        }

        .networks-section {
            margin: 3rem 0;
        }

        .networks-section h2 {
            font-size: 1.5rem;
            margin-bottom: 2rem;
            text-align: center;
            letter-spacing: 0.05em;
        }

        .networks-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
            gap: 2rem;
            margin: 2rem 0;
        }

        .network-card {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.75rem;
            padding: 1.5rem 1rem;
            background: #fafafa;
            border: 1px solid #000000;
            border-radius: 8px;
            transition: all 0.3s ease;
            text-align: center;
        }

        .network-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .network-card img {
            width: 48px;
            height: 48px;
            object-fit: contain;
        }

        .network-card .network-name {
            font-size: 0.9rem;
            font-weight: 600;
        }

        @media (max-width: 768px) {
            h1 {
                font-size: 2rem;
            }

            .logo {
                width: 180px;
                height: 180px;
            }

            .endpoints {
                padding: 1.5rem;
            }

            .endpoint-list .method {
                display: block;
                margin-bottom: 0.3rem;
            }

            .networks-grid {
                grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
                gap: 1.5rem;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <img src="/static/stake-capital-logo.jpg" alt="Stake Capital" class="logo" />

        <h1>STAKE CAPITAL</h1>

        <p class="tagline">
            x402 Facilitator – Payment verification and settlement infrastructure
        </p>

        <div class="networks-section">
            <h2>SUPPORTED NETWORKS</h2>
            <div class="networks-grid">
                <div class="network-card">
                    <img src="/static/avalanche.png" alt="Avalanche">
                    <span class="network-name">Avalanche</span>
                </div>
                <div class="network-card">
                    <img src="/static/solana.png" alt="Solana">
                    <span class="network-name">Solana</span>
                </div>
                <div class="network-card">
                    <img src="/static/polygon.png" alt="Polygon">
                    <span class="network-name">Polygon</span>
                </div>
                <div class="network-card">
                    <img src="/static/base.png" alt="Base">
                    <span class="network-name">Base</span>
                </div>
            </div>
        </div>

        <div class="endpoints">
            <h2>API ENDPOINTS</h2>
            <ul class="endpoint-list">
                <li><span class="method">GET</span> <code>/verify</code> – Supported verification schema</li>
                <li><span class="method">POST</span> <code>/verify</code> – Verify payment payload</li>
                <li><span class="method">GET</span> <code>/settle</code> – Supported settlement schema</li>
                <li><span class="method">POST</span> <code>/settle</code> – Settle payment on-chain</li>
                <li><span class="method">GET</span> <code>/supported</code> – List supported payment kinds</li>
                <li><span class="method">GET</span> <code>/health</code> – Health check</li>
            </ul>
        </div>

        <div class="footer">
            Powered by <a href="https://x402.org" target="_blank">x402 Protocol</a>
        </div>
    </div>
</body>
</html>"#;

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

/// `GET /supported`: Lists the x402 payment schemes and networks supported by this facilitator.
///
/// Facilitators may expose this to help clients dynamically configure their payment requests
/// based on available network and scheme support.
#[instrument(skip_all)]
pub async fn get_supported<A>(State(facilitator): State<A>) -> impl IntoResponse
where
    A: Facilitator,
    A::Error: IntoResponse,
{
    match facilitator.supported().await {
        Ok(supported) => (StatusCode::OK, Json(json!(supported))).into_response(),
        Err(error) => error.into_response(),
    }
}

#[instrument(skip_all)]
pub async fn get_health<A>(State(facilitator): State<A>) -> impl IntoResponse
where
    A: Facilitator,
    A::Error: IntoResponse,
{
    get_supported(State(facilitator)).await
}

/// `POST /verify`: Facilitator-side verification of a proposed x402 payment.
///
/// This endpoint checks whether a given payment payload satisfies the declared
/// [`PaymentRequirements`], including signature validity, scheme match, and fund sufficiency.
///
/// Responds with a [`VerifyResponse`] indicating whether the payment can be accepted.
#[instrument(skip_all)]
pub async fn post_verify<A>(
    State(facilitator): State<A>,
    Json(body): Json<VerifyRequest>,
) -> impl IntoResponse
where
    A: Facilitator,
    A::Error: IntoResponse,
{
    match facilitator.verify(&body).await {
        Ok(valid_response) => (StatusCode::OK, Json(valid_response)).into_response(),
        Err(error) => {
            tracing::warn!(
                error = ?error,
                body = %serde_json::to_string(&body).unwrap_or_else(|_| "<can-not-serialize>".to_string()),
                "Verification failed"
            );
            error.into_response()
        }
    }
}

/// `POST /settle`: Facilitator-side execution of a valid x402 payment on-chain.
///
/// Given a valid [`SettleRequest`], this endpoint attempts to execute the payment
/// via ERC-3009 `transferWithAuthorization`, and returns a [`SettleResponse`] with transaction details.
///
/// This endpoint is typically called after a successful `/verify` step.
#[instrument(skip_all)]
pub async fn post_settle<A>(
    State(facilitator): State<A>,
    Json(body): Json<SettleRequest>,
) -> impl IntoResponse
where
    A: Facilitator,
    A::Error: IntoResponse,
{
    match facilitator.settle(&body).await {
        Ok(valid_response) => (StatusCode::OK, Json(valid_response)).into_response(),
        Err(error) => {
            tracing::warn!(
                error = ?error,
                body = %serde_json::to_string(&body).unwrap_or_else(|_| "<can-not-serialize>".to_string()),
                "Settlement failed"
            );
            error.into_response()
        }
    }
}

fn invalid_schema(payer: Option<MixedAddress>) -> VerifyResponse {
    VerifyResponse::invalid(payer, FacilitatorErrorReason::InvalidScheme)
}

impl IntoResponse for FacilitatorLocalError {
    fn into_response(self) -> Response {
        let error = self;

        let bad_request = (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid request".to_string(),
            }),
        )
            .into_response();

        match error {
            FacilitatorLocalError::SchemeMismatch(payer, ..) => {
                (StatusCode::OK, Json(invalid_schema(payer))).into_response()
            }
            FacilitatorLocalError::ReceiverMismatch(payer, ..)
            | FacilitatorLocalError::InvalidSignature(payer, ..)
            | FacilitatorLocalError::InvalidTiming(payer, ..)
            | FacilitatorLocalError::InsufficientValue(payer) => {
                (StatusCode::OK, Json(invalid_schema(Some(payer)))).into_response()
            }
            FacilitatorLocalError::NetworkMismatch(payer, ..)
            | FacilitatorLocalError::UnsupportedNetwork(payer) => (
                StatusCode::OK,
                Json(VerifyResponse::invalid(
                    payer,
                    FacilitatorErrorReason::InvalidNetwork,
                )),
            )
                .into_response(),
            FacilitatorLocalError::ContractCall(..)
            | FacilitatorLocalError::InvalidAddress(..)
            | FacilitatorLocalError::ClockError(_) => bad_request,
            FacilitatorLocalError::DecodingError(reason) => (
                StatusCode::OK,
                Json(VerifyResponse::invalid(
                    None,
                    FacilitatorErrorReason::FreeForm(reason),
                )),
            )
                .into_response(),
            FacilitatorLocalError::InsufficientFunds(payer) => (
                StatusCode::OK,
                Json(VerifyResponse::invalid(
                    Some(payer),
                    FacilitatorErrorReason::InsufficientFunds,
                )),
            )
                .into_response(),
        }
    }
}
