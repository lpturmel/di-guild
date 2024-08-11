use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    let cookie = std::env::var("RAIDBOTS_COOKIE").expect("RAIDBOTS_COOKIE env var not set");
    let libsql_url = std::env::var("LIBSQL_URL").expect("Expected a url in the environment");
    let libsql_token = std::env::var("LIBSQL_TOKEN").expect("Expected a token in the environment");

    Config {
        cookie,
        libsql_url,
        libsql_token,
    }
});
#[non_exhaustive]
pub struct Config {
    /// RaidBots cookie
    pub cookie: String,
    pub libsql_url: String,
    pub libsql_token: String,
}
