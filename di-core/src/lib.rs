#![allow(clippy::new_ret_no_self)]
use error::{Error, Result};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod simc;

pub const RAIDBOTS_BASE_URL: &str = "https://www.raidbots.com";

#[derive(Debug, Clone)]
pub struct RaidBots {
    /// Cookie to authenticate with premium account
    cookie: Option<String>,
    http_client: reqwest::Client,
}

impl RaidBots {
    pub fn new() -> RaidBotsBuilder {
        RaidBotsBuilder::new()
    }
    fn build_request(&self, url: &str, method: reqwest::Method) -> reqwest::RequestBuilder {
        let mut req = self
            .http_client
            .request(method, url)
            .version(reqwest::Version::HTTP_2);
        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", format!("raidsid={}", cookie));
        }
        req
    }

    pub async fn create_sim(&self, sim_str: &str) -> Result<SimResponse> {
        let req: SimRequest = SimRequest {
            sim_type: SimType::Advanced,
            simc_version: SimCVersion::Weekly,
            advanced_input: sim_str.to_string(),
        };
        let url = format!("{}/sim", RAIDBOTS_BASE_URL);
        let req = self
            .build_request(&url, reqwest::Method::POST)
            .json(&req)
            .build()?;
        let res = self.http_client.execute(req).await?;
        let json = res.json::<SimResponse>().await?;
        Ok(json)
    }

    pub async fn get_report(&self, report_id: &str) -> Result<SimReportData> {
        let url = format!("{}/reports/{}/data.json", RAIDBOTS_BASE_URL, report_id);
        let req = self.build_request(&url, reqwest::Method::GET).build()?;
        let response = self.http_client.execute(req).await?;
        let json = response.json::<SimReportData>().await?;
        Ok(json)
    }

    pub async fn get_char(&self, name: &str, realm: &str) -> Result<serde_json::Value> {
        let url = format!(
            "{}/wowapi/character/us/{}/{}",
            RAIDBOTS_BASE_URL, realm, name
        );
        let req = self.build_request(&url, reqwest::Method::GET).build()?;
        let response = self.http_client.execute(req).await?;
        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }
}

pub struct RaidBotsBuilder {
    cookie: Option<String>,
    http_client: reqwest::Client,
}

impl RaidBotsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_cookie<S>(&mut self, cookie: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.cookie = Some(cookie.into());
        self
    }

    pub fn build(&self) -> RaidBots {
        RaidBots {
            cookie: self.cookie.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

impl Default for RaidBotsBuilder {
    fn default() -> Self {
        Self {
            cookie: None,
            http_client: reqwest::ClientBuilder::new()
                .use_rustls_tls()
                .build()
                .unwrap(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SimRequest {
    #[serde(rename = "type")]
    sim_type: SimType,
    #[serde(rename = "simcVersion")]
    simc_version: SimCVersion,
    advanced_input: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SimCVersion {
    Weekly,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SimType {
    Quick,
    Advanced,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimResponse {
    pub job_id: String,
    pub sim_id: String,
    pub simc_version: String,
    pub created: String,
    pub fight_length: usize,
    pub fight_style: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimDetailsRow {
    pub user_id: String,
    pub name: String,
    pub sim_str: String,
    pub added_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SqsWorkerPayload {
    /// The shared request id used to identify groups of messages
    pub request_id: String,
    pub user_id: String,
    #[serde(flatten)]
    pub sim_response: SimResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimReportData {
    pub sim: SimReportDataSim,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SimReportDataSim {
    pub players: Vec<SimReportDataSimPlayer>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SimReportDataSimPlayer {
    pub name: String,
    pub collected_data: SimReportDataSimPlayerCollectedData,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SimReportDataSimPlayerCollectedData {
    pub dps: SimReportDataSimPlayerCollectedDataDps,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SimReportDataSimPlayerCollectedDataDps {
    pub sum: f64,
    pub count: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub mean_variance: f64,
    pub mean_std_dev: f64,
}
