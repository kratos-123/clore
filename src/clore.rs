#[allow(dead_code)]
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use tracing::info;

use crate::clore::model::{market::Marketplace, wallet::Wallets};

use self::model::{resent::Resent, Card};

const HOST: &str = "https://api.clore.ai/";
const TOKEN: &str = "MGsjoMv1oxSly5.sJSMXHMehW3z1pdvO";
pub const SSH_PASSWORD: &str = "Hpcj08ZaOpCbTmn1Eu";
pub const JUPYTER_TOKEN: &str = "hoZluOjbCOQ5D5yH7R";
pub mod model;
pub struct Clore {}

impl Default for Clore {
    fn default() -> Self {
        Self {}
    }
}

impl Clore {
    pub async fn marketplace(&self) -> Result<Vec<Card>, String> {
        let url = format!("{}{}", HOST, "v1/marketplace");
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let markets = serde_json::from_str::<Marketplace>(&text)
            .map_err(|e| e.to_string())?
            .filter();
        info!("可用卡:{:?}", &markets);
        Ok(markets)
    }

    pub async fn wallet(&self) -> Result<f64, String> {
        let url = format!("{}{}", HOST, "v1/wallets");
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let wallets = text.parse::<Wallets>()?;
        let balance = wallets.filter();
        info!(text, balance);
        Ok(balance)
    }

    pub async fn create_order(&self, server_id: u32) {
        let url = format!("{}{}", HOST, "v1/create_order");
        Resent::new(server_id).to_string();
    }

    fn get_client() -> Result<Client, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("auth", HeaderValue::from_static(&TOKEN));
        ClientBuilder::new().default_headers(headers).build()
    }
}
