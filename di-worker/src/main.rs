use aws_lambda_events::event::sqs::SqsEventObj;
use di_core::SqsWorkerPayload;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::sync::Arc;

pub mod config;

async fn function_handler(event: HandlerEvent, shared_data: SharedData) -> Result<(), Error> {
    let data = event
        .payload
        .records
        .first()
        .ok_or(Error::from("No data in SQS event"))?;

    let report_data = shared_data
        .raidbots
        .get_report(&data.body.sim_response.sim_id)
        .await;
    let report_data = match report_data {
        Ok(report_data) => report_data,
        Err(e) => {
            tracing::error!("Error getting report data: {:?}", e);
            match e {
                di_core::error::Error::Reqwest(e) => match e.status() {
                    Some(status) if status.is_client_error() => {
                        let msg = "Report not found, the message will be retried...".to_string();
                        tracing::error!("{}", msg);
                        return Err(Error::from(msg));
                    }
                    _ => {
                        tracing::error!("Unexpected reqwest error getting report data: {:?}", e);
                        return Err(Error::from("Unexpected reqwest error getting report data"));
                    }
                },
                _ => {
                    tracing::error!("Unexpected error getting report data: {:?}", e);
                    return Err(Error::from("Error getting report data"));
                }
            }
        }
    };
    let conn = shared_data.db.clone().connect()?;
    let current_datetime = chrono::Utc::now();
    let formatted_datetime = current_datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let player_data = &report_data
        .sim
        .players
        .first()
        .ok_or(Error::from("No players in report"))?;
    let mean = player_data.collected_data.dps.mean;
    let _ = conn
        .execute(
            "INSERT INTO sim_results (request_id, dps_mean, completed_at, user_id, name) VALUES (?,?,?,?,?)",
            libsql::params![
                data.body.request_id.as_str(),
                mean,
                formatted_datetime,
                data.body.user_id.as_str(),
                player_data.name.to_lowercase(),
            ],
        )
        .await?;
    Ok(())
}

#[derive(Clone)]
struct SharedData {
    raidbots: di_core::RaidBots,
    db: Arc<libsql::Database>,
}

type HandlerEvent = LambdaEvent<SqsEventObj<SqsWorkerPayload>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();
    let config = &config::CONFIG;

    let db = libsql::Builder::new_remote(
        config.libsql_url.to_string(),
        config.libsql_token.to_string(),
    )
    .build()
    .await
    .expect("to build db");
    let shared_data = SharedData {
        raidbots: di_core::RaidBots::new().set_cookie(&config.cookie).build(),
        db: Arc::new(db),
    };
    run(service_fn(move |e: HandlerEvent| {
        function_handler(e, shared_data.clone())
    }))
    .await
}
