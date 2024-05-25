#[allow(dead_code)]
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

use self::model::{resent::Resent, Card};
use crate::server::clore::model::{market::Marketplace, wallet::Wallets};

pub const HOST: &str = "https://api.clore.ai/";
pub const TOKEN: &str = "2cFfpo5r18VwgEaArcPI_lYaVKm_-rXL";
pub const SSH_PASSWORD: &str = "lurq9SVzxGhpvok-Lkq8Lbb4iNI1u1vB";
pub const JUPYTER_TOKEN: &str = "hoZluOjbCOQ5D5yH7R";
pub const LOG_COLLECT_API: &str = "http://127.0.0.1:8888/printlnlog";
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
        // info!("服务器响应:{:?}", &text);
        let markets = serde_json::from_str::<Marketplace>(&text)
            .map_err(|e| e.to_string())?
            .filter();
        // info!("可用卡:{:?}", &markets);
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

    pub async fn create_order(&self, server_id: u32) -> Result<(), String> {
        let url = format!("{}{}", HOST, "v1/create_order");
        let body = Resent::new(server_id).to_string();
        let mut headers: HashMap<_, _> = HashMap::new();
        headers.insert("Content-type", HeaderValue::from_str("application/json"));
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let result = serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())?;
        let code = result.get("code").map_or("-1".to_string(), |val| {
            String::from(val.as_str().unwrap_or("-1"))
        });

        if code == "0" {
            Ok(())
        } else {
            Err(format!("创建服务器失败，错误码:{:?}", code))
        }
    }

    pub async fn my_orders() -> Result<(), String> {
        let url = format!("{}{}", HOST, "v1/my_orders");
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let result = serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())?;
        let code = result.get("code").map_or("-1".to_string(), |val| {
            String::from(val.as_str().unwrap_or("-1"))
        });

        todo!("获取到订单号")
    }

    pub async fn cancel_order(&self, order_id: u32) -> Result<(), String> {
        let url = format!("{}{}", HOST, "v1/cancel_order");
        let body = format!("\"{{\"id\":{}}}\"", order_id);
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let result = serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())?;
        let code = result.get("code").map_or("-1".to_string(), |val| {
            String::from(val.as_str().unwrap_or("-1"))
        });

        Ok(())
    }

    fn get_client() -> Result<Client, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("auth", HeaderValue::from_static(&TOKEN));
        ClientBuilder::new()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
    }
}
