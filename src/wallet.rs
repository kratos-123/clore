use chrono::{DateTime, Local, TimeZone};
use serde_json::error;

#[allow(dead_code)]
use std::io::{BufRead, BufReader, Error, Write};
use std::{clone, collections::HashMap, io::Read, net::IpAddr, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::clore::{model::wallet, Clore};

lazy_static::lazy_static! {
    pub static ref WALLETS_STATE:Arc<Mutex<Wallets>> = {
        Arc::new(Mutex::new(Wallets::default()))
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum AddressType {
    MASTER,
    SUB,
    NULL,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Wallet {
    pub address: String,
    pub sshaddr: Option<String>,
    pub sshport: Option<u32>,
    pub addr_type: AddressType,
    pub start_time: Option<DateTime<Local>>,
    pub order_id: Option<u32>,
    pub report_last_time: Option<DateTime<Local>>,
    pub deploy: Deployed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Deployed {
    NOTASSIGNED,
    DEPLOYING,
    DEPLOYED,
}

impl Wallet {
    pub fn new(address: String, addr_type: AddressType) -> Wallet {
        Wallet {
            address,
            addr_type,
            sshaddr: None,
            sshport: None,
            start_time: None,
            order_id: None,
            report_last_time: None,
            deploy: Deployed::NOTASSIGNED,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Wallets(pub HashMap<String, Wallet>);

impl std::ops::DerefMut for Wallets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Wallets {
    type Target = HashMap<String, Wallet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Wallets {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Wallets {
    async fn mstaddress(address: &str) -> AddressType {
        let url = "https://mainnet.nimble.technology/check_balance";
        let result = Wallets::curl(url, address).await;
        if let Err(_) = result {
            return AddressType::NULL;
        }
        let text = result.unwrap();
        if text.contains("Error") {
            AddressType::NULL
        } else {
            AddressType::MASTER
        }
    }

    async fn subaddress(address: &str) -> AddressType {
        let url = "https://mainnet.nimble.technology/register_particle";
        let result = Wallets::curl(url, address).await;
        if let Err(_) = result {
            return AddressType::NULL;
        }
        let text = result.unwrap();
        if text.contains("Task registered successfully") {
            AddressType::SUB
        } else {
            AddressType::NULL
        }
    }

    async fn curl(url: &str, address: &str) -> Result<String, String> {
        info!("网络请求:{},{}", url, address);
        let mut params = HashMap::new();
        params.insert("address", address);
        let client = reqwest::Client::new();
        let result = client
            .post(url)
            .json(&params)
            .send()
            .await
            .map_err(|e| e.to_string());
        if let Err(msg) = &result {
            warn!("发起网络请求失败:{}", msg);
            return Err(msg.clone());
        }

        let response = result.unwrap();

        if !&response.status().is_success() {
            let msg = format!("远程响应状态码不正确:{}", response.status());
            warn!(msg);
        }

        let result = response.text().await.map_err(|e| e.to_string());
        if let Err(msg) = &result {
            warn!(msg);
            return Err(msg.clone());
        }
        let text = &result.unwrap();
        info!("response:{:?}", text);
        Ok(text.to_string())
    }

    pub async fn load_address_file() -> Vec<Wallet> {
        let mut address = std::fs::File::open("./address.txt")
            .expect("文件:./address.txt不存在！，请创建此文件，");
        let mut addr = String::new();

        let _ = address.read_to_string(&mut addr);

        addr.split_inclusive('\n')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty() && !item.starts_with("#"))
            .map(|address| Wallet::new(address, AddressType::NULL))
            .collect::<Vec<Wallet>>()
    }

    pub async fn check(&mut self, other_wallets: &Vec<Wallet>) {
        for wallet in other_wallets.iter() {
            let address = wallet.address.clone();
            if (*self).contains_key(&address) {
                let wallet = self.get(&address).unwrap();
                info!(
                    "地址:{:?}已被检测过,地址角色:{:?}",
                    &address, wallet.addr_type
                );
                continue;
            }

            let (mstaddress, subaddress) =
                tokio::join!(Wallets::mstaddress(&address), Wallets::subaddress(&address));
            info!("地址检测结果:{:?},{:?}", mstaddress, subaddress);
            let addr_type = if let AddressType::MASTER = mstaddress {
                AddressType::MASTER
            } else if let AddressType::SUB = subaddress {
                AddressType::SUB
            } else {
                AddressType::NULL
            };
            if addr_type != AddressType::NULL {
                (*self).insert(
                    address.clone(),
                    Wallet::new(address.clone(), addr_type.clone()),
                );
            }
            info!("地址匹配结果:{:?}", addr_type.clone());
        }
    }

    // 过滤规则
    // 未分配订单id的服务器
    pub async fn filter(&self) -> Vec<Wallet> {
        let mut result: Vec<Wallet> = Vec::new();
        for (_, wallet) in (*self).iter() {
            if wallet.addr_type == AddressType::SUB && wallet.deploy == Deployed::NOTASSIGNED {
                result.push(wallet.clone());
            }
        }
        result
    }

    // 分配服务器
    pub async fn assgin_server(
        &mut self,
        wallet_adress: &str,
        order_id: u32,
        sshaddr: String,
        sshport: u32,
    ) -> Result<bool, String> {
        if !(*self).contains_key(wallet_adress) {
            return Err("不存在钱包地址！".to_string());
        }
        let local_time = Local::now();
        let wallet = (*self).get_mut(wallet_adress).unwrap();
        wallet.deploy = Deployed::DEPLOYING;
        wallet.start_time = Some(local_time);
        wallet.sshaddr = Some(sshaddr);
        wallet.sshport = Some(sshport);
        wallet.order_id = Some(order_id);

        Ok(true)
    }

    pub async fn update_log_collect_time(&mut self, wallet_adress: &str) -> bool {
        if !(*self).contains_key(wallet_adress) {
            return false;
        }
        let wallet = (*self).get_mut(wallet_adress).unwrap();
        let local_time = Local::now();
        wallet.report_last_time = Some(local_time);
        wallet.deploy = Deployed::DEPLOYED;
        true
    }

    // 超时未上报时间，则取消该机器订单号，重置所有钱包信息
    pub async fn filter_log_timeout(&mut self, clore: &Clore) {
        let mut order_ids: Vec<u32> = Vec::new();
        for (_, wallet) in (*self).iter_mut() {
            let nowtime = Local::now();

            if let Some(order_id) = wallet.order_id {
                // 上报时间若是超过了十分钟，则也取消，订单号
                if let Some(report_last_time) = wallet.report_last_time {
                    if nowtime.timestamp() - report_last_time.timestamp() > 10 * 60 {
                        order_ids.push(order_id);
                    }
                } else {

                    // 创建时间超过15分钟，还未有上报时间则，进行取消订单
                    if let Some(start_time) = wallet.start_time {
                        if nowtime.timestamp() - start_time.timestamp() > 15 * 60 {
                            order_ids.push(order_id);
                        }
                    }
                }
            }
        }
        for order_id in order_ids.iter() {
            let result = clore.cancel_order(order_id.clone()).await;
            if let Err(e) = result {
                error!("订单:{:?}取消失败,错误码：{:?}",order_id,e);
            }else {
                warn!("已取消{:?}该订单",order_id);
            }
        }

    }
}

pub async fn pool() {
    loop {
        let wallets = Arc::clone(&WALLETS_STATE);
        let mut row = wallets.lock().await;
        let other = Wallets::load_address_file().await;
        let wallets = row.filter().await;
        row.check(&other).await;
        warn!("未分配钱包{:?}",wallets);

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
