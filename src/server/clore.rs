use futures::executor::block_on;
#[allow(dead_code)]
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serde_json::{Number, Value};
use std::{collections::HashMap, sync::Arc};
use tracing::info;

use self::model::{resent::Resent, Card};
use crate::{
    config::{self, CONFIG},
    server::clore::model::{market::Marketplace, my_orders::MyOrders, wallet::Wallets},
};

pub mod model;
pub struct Clore {}

impl Default for Clore {
    fn default() -> Self {
        Self {}
    }
}

impl Clore {
    pub async fn marketplace(&self) -> Result<Vec<Card>, String> {
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/marketplace");
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
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/wallets");
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
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/create_order");
        let body = Resent::new(server_id);
        info!("body:{}", serde_json::to_string(&body).unwrap());
        return Ok(());
        let mut headers: HashMap<_, _> = HashMap::new();
        headers.insert("Content-type", HeaderValue::from_str("application/json"));
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
        info!("{:?}", &text);
        let result = serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())?;
        let code = result.get("code").map_or(-1i64, |val| {
            val.as_number()
                .unwrap_or(&Number::from(-1))
                .as_i64()
                .unwrap_or(-1)
        });

        if code == 0 {
            Ok(())
        } else {
            Err(format!("创建服务器失败，错误码:{:?}", code))
        }
    }

    pub async fn my_orders(&self) -> Result<(), String> {
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/my_orders");
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        info!("my_order_text:{:?}", text);
        let result = serde_json::from_str::<MyOrders>(&text).map_err(|e| e.to_string());
        info!("获取到订单号:{:?}", result);
        Ok(())
    }

    pub async fn cancel_order(&self, order_id: u32) -> Result<(), String> {
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/cancel_order");
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
        let config::Clore { api_token, .. } = block_on(Clore::get_config());
        let token = api_token.clone();
        let mut headers = HeaderMap::new();
        headers.insert("auth", HeaderValue::from_str(&token).unwrap());
        ClientBuilder::new()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
    }

    async fn get_config() -> config::Clore {
        let mutex_conf = Arc::clone(&CONFIG);
        let config = &mutex_conf.lock().await;
        (*config).clore.clone()
    }
}
