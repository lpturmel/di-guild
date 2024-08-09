use std::sync::Arc;

use self::config::CONFIG;
use aws_config::BehaviorVersion;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::{routing::post, Router};
use handlers::webhook;
use tower_http::trace::TraceLayer;

pub mod commands;
pub mod config;
pub mod discord;
pub mod error;
pub mod handlers;
pub mod mw;

#[derive(Clone)]
pub struct AppState {
    pub sqs_client: aws_sdk_sqs::Client,
    pub db: Arc<libsql::Database>,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();
    let trace_layer =
        TraceLayer::new_for_http().on_request(|_: &Request<Body>, _: &tracing::Span| {
            tracing::info!(message = "begin request!")
        });

    let db = libsql::Builder::new_remote(
        CONFIG.libsql_url.to_string(),
        CONFIG.libsql_token.to_string(),
    )
    .build()
    .await
    .expect("to build db");
    // let conn = db.connect().expect("to connect to db");

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let sqs_sdk = aws_sdk_sqs::Client::new(&aws_config);
    let state = AppState {
        sqs_client: sqs_sdk,
        db: Arc::new(db),
    };
    let app = Router::new()
        .route("/webhook", post(webhook))
        .layer(from_fn(mw::validate_req))
        .layer(trace_layer)
        .with_state(state);

    #[cfg(debug_assertions)]
    {
        dotenv::dotenv().ok();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
        tracing::debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        // To run with AWS Lambda runtime, wrap in our `LambdaLayer`
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }
}
