use aws_lambda_events::event::sqs::SqsEventObj;
use di_core::SimResponse;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

async fn function_handler(event: HandlerEvent, shared_data: SharedData) -> Result<(), Error> {
    let data = event
        .payload
        .records
        .first()
        .ok_or(Error::from("No data in SQS event"))?;
    Ok(())
}

#[derive(Clone)]
struct SharedData {
    http_client: reqwest::Client,
}

impl SharedData {
    fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }
}

type HandlerEvent = LambdaEvent<SqsEventObj<SimResponse>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let shared_data = SharedData::new();
    run(service_fn(move |e: HandlerEvent| {
        function_handler(e, shared_data.clone())
    }))
    .await
}
