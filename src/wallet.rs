#[allow(dead_code)]
use std::io::{BufRead, BufReader, Error, Write};
use std::{clone, collections::HashMap, io::Read, net::IpAddr, sync::Arc};
use tokio::sync::Mutex;
use tracing::{info, warn};

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

#[derive(Debug, PartialEq)]
pub struct Wallet {
    address: String,
    remoteip: Option<IpAddr>,
    addr_type: AddressType,
}

impl Wallet {
    pub fn new(address: String, addr_type: AddressType) -> Wallet {
        Wallet {
            address,
            remoteip: None,
            addr_type,
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
        // let wallets = Arc::clone(&WALLETS_STATE);
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
}

pub async fn pool() {
    loop {
        let wallets = Arc::clone(&WALLETS_STATE);
        let mut row = wallets.lock().await;
        let other = Wallets::load_address_file().await;
        row.check(&other).await;

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
