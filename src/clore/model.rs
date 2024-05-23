use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, PartialEq, EnumString)]
pub enum CardType {
    NVIDIA4090,
    NVIDIA4080S,
    NVIDIA4080,
    NVIDIA4070S,
    NVIDIA4070,
    NVIDIA4070TI,
    NVIDIA3090,
    NVIDIA3090TI,
    NVIDIA3080TI,
    NVIDIA3080,
    NVIDIA1080TI,
    NVIDIA1080,
    UNKNOWN(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    BITCOIN,
    CLORE,
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bitcoin" => Ok(Currency::BITCOIN),
            "CLORE-Blockchain" => Ok(Currency::CLORE),
            _ => Err("没有此货币".to_string()),
        }
    }
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        match &self {
            Currency::BITCOIN => "bitcoin".to_string(),
            Currency::CLORE => "CLORE-Blockchain".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Card {
    pub server_id: u32,
    pub price_demand: f64,
    pub price_spot: f64,
    pub mrl: u32,
    pub card_number: i32,
    pub rented: bool,
    pub card_type: CardType,
}

pub mod market {
    use std::{
        collections::HashMap,
        ops::{Deref, DerefMut},
        str::FromStr,
    };

    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use tracing::{info, warn};

    use super::{Card, CardType};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Net {
        up: f64,
        down: f64,
        cc: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Specs {
        pub mb: String,
        pub cpu: String,
        pub cpus: String,
        pub ram: f64,
        pub disk: String,
        pub disk_speed: f32,
        pub gpu: String,
        pub gpuram: f32,
        pub net: Net,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Price {
        pub on_demand: HashMap<String, f64>,
        pub spot: HashMap<String, f64>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Server {
        pub allowed_coins: Vec<String>,
        pub id: u32,
        pub owner: u32,
        pub mrl: u32,
        pub price: Price,
        pub rented: bool,
        pub specs: Specs,
        pub rating: HashMap<String, f32>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Marketplace {
        servers: Vec<Server>,
        my_servers: Vec<u32>,
        code: u32,
    }

    impl Marketplace {
        pub fn filter(&self) -> Vec<Card> {
            let regex = Regex::new(r"(1080|3080|3090|4070|4080|4080|4090)").unwrap();
            let cards: Vec<Card> = (*self)
                .iter()
                .filter(|item| {
                    let machine_properties = &item.specs;
                    let gpu = &machine_properties.gpu;
                    regex.is_match(&gpu)
                        && item.rating.get("avg").unwrap() > &3.5f32
                        && item.allowed_coins.contains(&"CLORE-Blockchain".to_string())
                        && item.rented
                })
                .map(|itme| {
                    let card_info = itme
                        .specs
                        .gpu
                        .split(' ')
                        .map(|item| item.to_string())
                        .collect::<Vec<String>>();
                    let number = card_info.get(0).map_or(0, |s| {
                        let s = s.replace("x", "");
                        s.parse::<i32>().unwrap_or_default()
                    });
                    let factory = card_info
                        .get(1)
                        .map(|item| item.to_owned())
                        .unwrap_or_default();
                    let card_type = card_info
                        .get(4)
                        .map(|item| item.to_owned())
                        .unwrap_or_default();
                    let mut flag = card_info
                        .get(5)
                        .map(|itme| itme.to_uppercase().to_owned())
                        .unwrap_or_default();
                    flag = match flag.as_str() {
                        "TI" => "TI".to_string(),
                        "SUPER" => "S".to_string(),
                        _ => "".to_string(),
                    };
                    let card_type =
                        CardType::from_str(&format!("{}{}{}", factory, card_type, flag))
                            .unwrap_or_else(|_| CardType::UNKNOWN(card_info.join(" ")));
                    let card = Card {
                        server_id: itme.id,
                        price_demand: itme
                            .price
                            .on_demand
                            .get("CLORE-Blockchain")
                            .and_then(|price| Some(price.clone()))
                            .unwrap_or_default(),
                        price_spot: itme
                            .price
                            .spot
                            .get("CLORE-Blockchain")
                            .and_then(|price| Some(price.clone()))
                            .unwrap_or_default(),
                        mrl: itme.mrl,
                        card_number: number,
                        rented: itme.rented,
                        card_type: card_type,
                    };
                    card
                })
                .filter(|item| {
                    if let CardType::UNKNOWN(_) = item.card_type {
                        warn!("未知显卡:{:?}", item.card_type);
                        false
                    } else {
                        true
                    }
                })
                .collect();
            info!("显卡数量:{:?}", cards);
            cards
        }
    }

    impl Deref for Marketplace {
        type Target = Vec<Server>;

        fn deref(&self) -> &Self::Target {
            &self.servers
        }
    }

    impl DerefMut for Marketplace {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.servers
        }
    }
}

pub mod wallet {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Wallet {
        pub name: String,
        pub deposit: String,
        pub balance: f64,
        pub withdrawal_fee: f64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Wallets {
        pub wallets: Vec<Wallet>,
        pub code: i32,
    }

    impl FromStr for Wallets {
        type Err = String;

        fn from_str(s: &str) -> Result<Wallets, Self::Err> {
            let wallets = serde_json::from_str::<Wallets>(s).map_err(|e| e.to_string())?;
            Ok(wallets)
        }
    }

    impl Wallets {
        pub fn filter(&self) -> f64 {
            let mut balance = 0f64;
            for wallet in self.wallets.iter() {
                if wallet.name == "CLORE-Blockchain" {
                    balance = wallet.balance;
                    break;
                }
            }
            balance
        }
    }
}

pub mod resent {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::clore::{JUPYTER_TOKEN, SSH_PASSWORD};

    use super::Currency;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resent {
        currency: Currency,
        image: String,
        renting_server: u32,
        #[serde(rename(serialize = "type"))]
        demand: String,
        ports: HashMap<String, String>,
        env: HashMap<String, String>,
        jupyter_token: String,
        ssh_password: String,
        command: String,
    }

    impl Resent {
        pub fn new(server_id: u32) -> Resent {
            let mut ports = HashMap::<String, String>::new();
            ports.insert("22".to_string(), "tcp".to_string());
            ports.insert("8888".to_string(), "http".to_string());
            let command = r##"#!/bin/sh
            apt update -y 
            apt install git -y
            mkdir $HOME/nimble && cd $HOME/nimble && git clone https://github.com/victor-vb/clore.git
            cd $HOME/nimble/clore && chmod +x ./env.sh && ./env.sh
            "##;
            Self {
                currency: Currency::CLORE,
                image: "cloreai/torch:2.0.1".to_string(),
                renting_server: server_id,
                demand: "on-demand".to_string(),
                ports: ports,
                env: Default::default(),
                jupyter_token: JUPYTER_TOKEN.to_string(),
                ssh_password: SSH_PASSWORD.to_string(),
                command: command.to_string(),
            }
        }
    }

    impl ToString for Resent {
        fn to_string(&self) -> String {
            serde_json::to_string(&self).unwrap_or_default()
        }
    }
}
