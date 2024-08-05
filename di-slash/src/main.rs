use axum::body::Body;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::{routing::post, Router};
use handlers::webhook;
use tower_http::trace::TraceLayer;

pub mod commands;
pub mod discord;
pub mod error;
pub mod handlers;
pub mod mw;

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
    let app = Router::new()
        .route("/webhook", post(webhook))
        .layer(from_fn(mw::validate_req))
        .layer(trace_layer);

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
