use futures::executor::block_on;
use model::resent::ResentWeb;
#[allow(dead_code)]
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serde_json::{Number, Value};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{Read, Write},
    sync::Arc,
};
use tracing::{error, info};

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
        info!("获取市场数据");
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
        let mut file = File::open(env::current_dir().unwrap().join("market.json")).unwrap();
        let _ = file.write_all(text.as_bytes());
        // info!("服务器响应:{:?}", &text);

        let markets = serde_json::from_str::<Marketplace>(&text)
            .map_err(|e| e.to_string())?
            .filter()
            .iter()
            .map(|card| card.clone())
            .collect::<Vec<_>>();
        info!("可用卡:{:?}", &markets);
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

    pub async fn create_order(&self, card: &Card, address: Vec<String>) -> Result<(), String> {
        let config::Clore {
            api_host,
            ssh_passwd,
            command,
            ..
        } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/create_order");
        let command = command
            .replace("{server_id}", card.server_id.to_string().as_str())
            .replace("{card_number}", card.card_number.to_string().as_str())
            .replace("{address}", address.join("-").as_str());
        let mut resent = Resent::new(card.server_id, ssh_passwd, command);
        let env = &mut resent.env;
        env.insert("SERVER_ID".to_string(), card.server_id.to_string());
        env.insert("CARD_NUMBER".to_string(), card.card_number.to_string());
        env.insert("ADDRESS".to_string(), address.join("-"));
        info!("body:{}", serde_json::to_string(&resent).unwrap());
        let mut headers: HashMap<_, _> = HashMap::new();
        headers.insert("Content-type", HeaderValue::from_str("application/json"));
        let text = Clore::get_client()
            .map_err(|e| e.to_string())?
            .post(url)
            .json(&resent)
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

    pub async fn create_order_web_api(
        &self,
        card: &Card,
        address: Vec<String>,
    ) -> Result<(), String> {
        let config::Clore {
            web_api_host,
            web_token,
            ssh_passwd,
            command,
            ..
        } = Clore::get_config().await;
        let url = format!("{}{}", web_api_host, "webapi/create_order");
        let command = command
            .replace("{server_id}", card.server_id.to_string().as_str())
            .replace("{card_number}", card.card_number.to_string().as_str())
            .replace("{address}", address.join("-").as_str());
        let mut resent = ResentWeb::new(card.server_id, ssh_passwd, web_token, command.clone());
        let env = &mut resent.env;
        env.insert("SERVER_ID".to_string(), card.server_id.to_string());
        env.insert("CARD_NUMBER".to_string(), card.card_number.to_string());
        env.insert("ADDRESS".to_string(), address.join("-"));
        info!("resent:{:?}", resent);

        let client = Clore::get_client().map_err(|e| e.to_string())?;
        info!("command:{:?}", command.clone());
        let text = client
            .post(url)
            .json(&resent)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        info!("{}", text);
        if text.contains("completed") {
            info!("下单成功！");
            Ok(())
        } else {
            error!("下单失败:{:?}", text);
            Err(text)
        }
    }

    pub async fn my_orders(&self) -> Result<MyOrders, String> {
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
        // info!("my_order_text:{}", text);
        let result: Result<MyOrders, String> =
            serde_json::from_str::<MyOrders>(&text).map_err(|e| e.to_string());
        if let Ok(my_orders) = &result {
            info!("获取到订单号:\n{}", my_orders);
        } else {
            error!("获取订单失败:{:?}", result);
        }

        result
    }

    pub async fn cancel_order(&self, order_id: u32) -> Result<(), String> {
        let config::Clore { api_host, .. } = Clore::get_config().await;
        let url = format!("{}{}", api_host, "v1/cancel_order");
        let body = format!(r#"{{"id":"{}"}}"#, order_id);
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        let text = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| e.to_string())?
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        info!("cancel_order:{}", text);
        let result = serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())?;
        let code = result.get("code").map_or(-1i64, |val| {
            val.as_number()
                .unwrap_or(&Number::from(-1))
                .as_i64()
                .unwrap_or(-1)
        });

        if code == 0 {
            info!("取消订单成功");
            Ok(())
        } else {
            let message = format!("取消失败:{:?}", code);
            error!("{}", message);
            Err(message)
        }
    }

    pub async fn cancel_order_web_api(&self, order_id: u32) -> Result<(), String> {
        let config::Clore {
            web_api_host,
            web_token,
            ..
        } = Clore::get_config().await;
        let body = format!(
            r#"{{"id":{},"rating":2,"token":"{}"}}"#,
            order_id, web_token
        );
        let url = format!("{}webapi/marketplace/cancel_order", web_api_host);
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        let result = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| e.to_string())?
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        if r#"{"status":"ok"}"# == &result {
            info!("订单取消成功:{}", result);
            Ok(())
        } else {
            error!("取消失败:{}", result);
            Err(result)
        }
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

    pub async fn get_config() -> config::Clore {
        let mutex_conf = Arc::clone(&CONFIG);
        let config = &mutex_conf.lock().await;
        (*config).clore.clone()
    }

    pub fn import_block_server_ids() -> Vec<u32> {
        let mut black_server_ids = Vec::<u32>::new();
        let openfile = Clore::open_block_file();
        if openfile.is_err() {
            error!("无法导入拉黑文件!");
            return black_server_ids;
        }
        let mut ids = String::from("");
        let mut reader = std::io::BufReader::new(openfile.unwrap());
        let _ = reader.read_to_string(&mut ids);
        black_server_ids = ids
            .split("\n")
            .into_iter()
            .map(|item| item.trim().parse::<u32>().unwrap_or_default())
            .collect::<Vec<u32>>();
        info!("黑名单:{:?}", black_server_ids);
        black_server_ids
    }

    pub fn append_block_server_id(server_id: u32) -> bool {
        let block_server_ids = Clore::import_block_server_ids();

        let openfile = Clore::open_block_file();
        if openfile.is_err() {
            error!("无法导出拉黑文件!");
            return false;
        }
        if !block_server_ids.contains(&server_id) {
            let mut writer = std::io::BufWriter::new(openfile.unwrap());
            let _ = writer.write_all(format!("\n{}", server_id).as_bytes());
        }
        true
    }

    fn open_block_file() -> Result<File, String> {
        let dir = std::env::current_dir().map_err(|e| e.to_string())?;
        let file = dir.join("block_server_ids.txt");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(true)
            .open(file)
            .map_err(|e| e.to_string())?;
        Ok(file)
    }
}
