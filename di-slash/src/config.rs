use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    let cookie = std::env::var("COOKIE").expect("COOKIE env var not set");
    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL env var not set");
    Config { cookie, queue_url }
});
#[non_exhaustive]
pub struct Config {
    /// RaidBots cookie
    pub cookie: String,
    /// AWS SQS queue url
    pub queue_url: String,
}
