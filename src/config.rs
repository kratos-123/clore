use std::{
    any::{self, Any},
    fs::OpenOptions,
    io::{BufReader, BufWriter, Read, Write},
    net::IpAddr,
    sync::Arc,
};

use futures::executor::block_on;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{error, info};

lazy_static! {
    pub static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::new()));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub address: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monitor {
    pub api_report_log: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub ip: Option<IpAddr>,
    pub port: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clore {
    pub web_api_host: String,
    pub web_token: String,
    pub api_host: String,
    pub api_token: String,
    pub ssh_passwd: String,
    pub command: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub wallet: Wallet,
    pub monitor: Monitor,
    pub server: Server,
    pub clore: Clore,
}

impl Config {
    pub fn new() -> Config {
        let result = Config::import_config();
        assert_eq!(
            any::TypeId::of::<Result<Config, String>>(),
            result.type_id()
        );
        result.unwrap()
    }

    pub fn import_config() -> Result<Config, String> {
        let result = std::env::current_dir();
        match result {
            Ok(path) => {
                let config_flie = path.join(".conf.toml");
                let flie = std::fs::File::open(config_flie).map_err(|e| e.to_string())?;
                let mut buf = String::new();
                let mut reader = BufReader::new(flie);
                let _ = reader.read_to_string(&mut buf);

                let config = toml::from_str::<Config>(&buf).map_err(|e| e.to_string())?;

                info!("{:?}", config);

                // info!("{:?}",builder);
                Ok(config)
            }
            Err(ref e) => {
                error!("{:?}", e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub fn export_config(any: impl Serialize) -> Result<(), String> {
        let result = std::env::current_dir();
        match result {
            Ok(path) => {
                let config_flie = path.join(".conf.bak.toml");
                let file = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(config_flie)
                    .map_err(|e| e.to_string())?;

                let mut writer: BufWriter<_> = BufWriter::new(file);
                let string = toml::to_string(&any).unwrap();
                let _ = writer.write_all(&string.as_bytes());
                Ok(())
            }
            Err(ref e) => {
                error!("{:?}", e.to_string());
                Err(e.to_string())
            }
        }
    }
}
