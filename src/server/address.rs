
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, sync::Arc};
use strum::Display;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{config::CONFIG, server::clore::Clore};

use super::{
    clore::model::CardType,
    ssh,
};

lazy_static::lazy_static! {
    pub static ref WALLETS_STATE:Arc<Mutex<Address>> = {
        Arc::new(Mutex::new(Address::default()))
    };
}

#[derive(Debug, Display, PartialEq, Clone, Serialize, Deserialize)]
pub enum AddressType {
    MASTER,
    SUB,
    NULL,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: f64,
    pub addr_type: AddressType,
    pub start_time: Option<DateTime<Local>>,
    pub report_last_time: Option<DateTime<Local>>,
    pub deploy: Deployed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Display)]
pub enum Deployed {
    NOTASSIGNED,
    DEPLOYING {
        orderid: u32,
        serverid: u32,
        sshaddr: Option<String>,
        sshport: Option<u16>,
    },
    DEPLOYED {
        orderid: u32,
        serverid: u32,
        sshaddr: Option<String>,
        sshport: Option<u16>,
    },
}

impl Wallet {
    pub fn new(address: String, addr_type: AddressType) -> Wallet {
        Wallet {
            address,
            addr_type,
            balance: 0f64,
            start_time: None,
            report_last_time: None,
            deploy: Deployed::NOTASSIGNED,
        }
    }

    pub fn set_balance(&mut self, balance: f64) -> bool {
        if self.addr_type == AddressType::MASTER {
            self.balance = balance;
            return true;
        } else {
            return false;
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Address(pub HashMap<String, Wallet>);

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str("\n");
        for (address, wallet) in (*self).iter() {
            if wallet.addr_type == AddressType::MASTER {
                let row: String = format!(
                    "addr:{},type:{},balance:{}\n",
                    address, wallet.addr_type, wallet.balance
                );
                let _ = f.write_str(&row);
            }
        }
        for (address, wallet) in (*self).iter() {
            if wallet.addr_type != AddressType::MASTER {
                let row: String = format!(
                    "addr:{},type:{},deploy:{}\n",
                    address, wallet.addr_type, wallet.deploy
                );
                let _ = f.write_str(&row);
            }
        }
        Ok(())
    }
}

impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Address {
    type Target = HashMap<String, Wallet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Address {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Address {
    async fn mstaddress(address: &str) -> Option<f64> {
        let url = "https://mainnet.nimble.technology/check_balance";
        let result = Address::curl(url, address).await;
        if let Err(_) = result {
            return None;
        }
        let text = result.unwrap();
        let reg = regex::Regex::new(r"Total: ([\d\.]+)").unwrap();

        match reg.captures(&text) {
            Some(captures) => {
                let (_, [balance, ..]) = captures.extract::<1>();
                balance.parse::<f64>().ok()
            }
            None => None,
        }
    }

    #[allow(dead_code)]
    async fn subaddress(address: &str) -> AddressType {
        let url = "https://mainnet.nimble.technology/register_particle";
        let result = Address::curl(url, address).await;
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

        info!("远程响应状态码:{}", response.status());

        let result = response.text().await.map_err(|e| e.to_string());
        if let Err(msg) = &result {
            warn!(msg);
            return Err(msg.clone());
        }
        let text = &result.unwrap();
        info!("远程响应结果:{}", text);
        Ok(text.to_string())
    }

    pub async fn load_address_file(&self) -> crate::config::Address {
        let mutex_conf = Arc::clone(&CONFIG);
        let config = &mutex_conf.lock().await;
        config.address.clone()
    }

    pub async fn check(&mut self, other_wallets: &crate::config::Address) {
        for mst_address in other_wallets.mst_address.iter() {
            if !(*self).contains_key(mst_address) {
                (*self).insert(
                    mst_address.clone(),
                    Wallet::new(mst_address.clone(), AddressType::MASTER),
                );
            }
            if let Some(balance) = Address::mstaddress(&mst_address).await {
                self.get_mut(mst_address).unwrap().set_balance(balance);
            }
        }

        for sub_address in other_wallets.sub_address.iter() {
            if !(*self).contains_key(sub_address) {
                (*self).insert(
                    sub_address.clone(),
                    Wallet::new(sub_address.clone(), AddressType::SUB),
                );
            }
        }
    }

    /// 获取没有分配的挖矿地址
    pub async fn get_unusd_wallet(&mut self) -> Vec<Wallet> {
        let mut wallets: Vec<Wallet> = Vec::new();
        let result = Clore::default().my_orders().await;

        if let Ok(my_orders) = result {
            let orders = (*my_orders).clone();
            // 过滤掉已经知道的serverid和钱包地址，已经知道的订单对应的不去链接ssh获取挖矿进程
            // 如果有对应的serverid但是orderid 为零，则补充上相关信息
            // 提取当钱包所有已知的serverid
            let mut serverids = Vec::new();
            for (_, wallet) in (*self).iter_mut() {
                match wallet.deploy {
                    Deployed::NOTASSIGNED => {}
                    Deployed::DEPLOYING {
                        orderid, serverid, ..
                    } => {
                        for order in orders.iter() {
                            if order.server_id == serverid && orderid == 0 {
                                wallet.deploy = Deployed::DEPLOYING {
                                    orderid: order.order_id,
                                    serverid: order.server_id,
                                    sshaddr: order.get_ssh_host(),
                                    sshport: order.get_map_ssh_port(),
                                };
                                serverids.push(order.server_id);
                            }
                        }
                    }
                    Deployed::DEPLOYED { serverid, .. } => {
                        for order in orders.iter() {
                            if order.server_id == serverid {
                                serverids.push(order.server_id);
                            }
                        }
                    }
                };
            }

            let mut filter_orders = Vec::new();
            for order in orders.iter() {
                if !serverids.contains(&order.server_id) {
                    filter_orders.push(order.clone());
                }
            }

            // 尝试远程ssh执行获取python挖矿进程。如果链接有出问题，则取消本次更新
            let (lists, error) = ssh::Ssh::try_run_command_remote(&filter_orders).await;
            // 对ssh获取成功的进程，将服务器信息挂在到子钱包地址上去
            for (wallet_adress, deployed) in lists {
                let _ = (*self).assgin_server(&wallet_adress, deployed).await;
            }

            // 等待所有服务器成功挂载以后，在看有需要部署的没
            if !error.is_empty() {
                return wallets;
            }
        }

        // 返回没有租用服务器的钱包地址
        for (_, wallet) in (*self).iter_mut() {
            if wallet.addr_type == AddressType::SUB && wallet.deploy == Deployed::NOTASSIGNED {
                wallets.push(wallet.clone());
            }
        }

        wallets
    }

    async fn resent_server(&mut self, wallets: Vec<Wallet>) {
        if wallets.len() > 0 {
            let clore = Clore::default();
            let markets = clore.marketplace().await;
            let wallets = wallets.as_slice().chunks(2);
            for wallet in wallets {
                let address = wallet
                    .iter()
                    .map(|item| item.address.clone())
                    .collect::<Vec<String>>();
                if let Ok(cards) = &markets {
                    for card in cards.iter() {
                        if card.card_type == CardType::NVIDIA4090
                            && card.card_number as usize == wallet.len()
                        {
                            info!(
                                "显卡租用中,服务器显卡数量:{},显卡型号:{}",
                                card.card_number, card.card_type
                            );
                            let _ = clore.create_order_web_api(card, address.clone()).await;
                            for wallet_adress in address.iter() {
                                let _ = self
                                    .assgin_server(
                                        wallet_adress,
                                        Deployed::DEPLOYING {
                                            orderid: 0,
                                            serverid: card.server_id,
                                            sshaddr: None,
                                            sshport: None,
                                        },
                                    )
                                    .await;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // 分配服务器
    pub async fn assgin_server(
        &mut self,
        wallet_adress: &str,
        deploy: Deployed,
    ) -> Result<(), String> {
        if !(*self).contains_key(wallet_adress) {
            return Err("不存在钱包地址！".to_string());
        }
        let local_time = Local::now();
        let wallet = (*self).get_mut(wallet_adress).unwrap();

        if Deployed::NOTASSIGNED == wallet.deploy {
            wallet.deploy = deploy;
            wallet.start_time = Some(local_time);
        }
        Ok(())
    }

    pub async fn update_log_collect_time(&mut self, wallet_adress: &str) -> bool {
        if !(*self).contains_key(wallet_adress) {
            return false;
        }
        let wallet = (*self).get_mut(wallet_adress).unwrap();
        if let Deployed::DEPLOYING {
            orderid,
            serverid,
            sshaddr,
            sshport,
        } = &wallet.deploy
        {
            let local_time = Local::now();
            wallet.report_last_time = Some(local_time);
            wallet.deploy = Deployed::DEPLOYED {
                orderid: orderid.clone(),
                serverid: serverid.clone(),
                sshaddr: sshaddr.clone(),
                sshport: sshport.clone(),
            };
        }

        true
    }

    // 超时未上报时间，则取消该机器订单号，重置所有钱包信息
    pub async fn filter_log_timeout(&mut self, clore: &Clore) {
        let mut order_ids: Vec<u32> = Vec::new();
        for (_, wallet) in (*self).iter_mut() {
            let nowtime = Local::now();
            match &wallet.deploy {
                Deployed::NOTASSIGNED => {}
                Deployed::DEPLOYING { orderid, .. } => {
                    // 创建时间超过25分钟，还未有上报时间则，进行取消订单
                    if let Some(start_time) = wallet.start_time {
                        if nowtime.timestamp() - start_time.timestamp() > 25 * 60 {
                            if orderid != &0 {
                                order_ids.push(orderid.clone());
                            }
                        }
                    }
                }
                Deployed::DEPLOYED { orderid, .. } => {
                    // 上报时间若是超过了十分钟，则也取消，订单号
                    if let Some(report_last_time) = wallet.report_last_time {
                        if nowtime.timestamp() - report_last_time.timestamp() > 10 * 60 {
                            if orderid != &0 {
                                order_ids.push(orderid.clone());
                            }
                        }
                    }
                }
            }
        }
        for order_id in order_ids.iter() {
            let result = clore.cancel_order(order_id.clone()).await;
            if let Err(e) = result {
                error!("订单:{:?}取消失败,错误码：{:?}", order_id, e);
            } else {
                warn!("已取消{:?}该订单", order_id);
            }
        }
    }
}

pub async fn pool() {
    loop {
        let wallets = Arc::clone(&WALLETS_STATE);
        let mut locked = wallets.lock().await;
        let other = Address::default().load_address_file().await;
        locked.check(&other).await;
        let wallets = locked.get_unusd_wallet().await;
        info!("当前绑定信息:{}", *locked);
        locked.resent_server(wallets).await;

        // let address = wallets
        //     .iter()
        //     .map(|wallet| wallet.address.to_string())
        //     .collect::<Vec<String>>();

        // if wallets.len() > 0 {
        //     warn!("待分配地址:\n{}", address.join("\n"));
        //     let market = Clore::default().marketplace().await;
        //     if let Ok(cards) = market {
        //         let server_ids = cards
        //             .iter()
        //             .filter(|item| item.card_number == 2)
        //             .map(|item| item.server_id)
        //             .collect::<Vec<u32>>();
        //         info!("server_ids:{:?}", server_ids);
        //     }
        // }
        // drop(locked);
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        // tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
