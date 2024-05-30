use lazy_static::lazy_static;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::config::CONFIG;
use crate::monitor::log::read_log_file;

use self::log::Logs;
use self::nvidia::GeForces;
pub mod log;
pub mod nvidia;
pub mod process;
lazy_static! {
    pub static ref MONITOR: Arc<Mutex<Monitor>> = Arc::new(Mutex::new(Monitor::new()));
    pub static ref LOG: Arc<Mutex<(UnboundedSender<LogChannel>, UnboundedReceiver<LogChannel>)>> =
        Arc::new(Mutex::new(unbounded_channel::<LogChannel>()));
}

#[derive(Debug,Clone)]
pub struct LogChannel{
    filename:String,
    body:String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monitor {
    server_id: Option<u32>,
    address: Vec<String>,
    nvidias: GeForces,
    logs: Logs,
    upload_log: HashMap<String, Vec<String>>,
}

impl Monitor {
    fn new() -> Monitor {
        Monitor {
            server_id: Monitor::get_server_id(),
            address: Monitor::get_address(),
            nvidias: GeForces::new(),
            logs: Logs::new(),
            upload_log: HashMap::<String, Vec<String>>::new(),
        }
    }

    fn get_server_id() -> Option<u32> {
        let server_id = std::env::var("SERVER_ID")
            .map_err(|e| e.to_string())
            .and_then(|server_id| server_id.parse::<u32>().map_err(|e| e.to_string()))
            .ok();
        if server_id.is_none() {
            error!("无法从环境变量中获取当前SERVER_ID")
        }
        server_id
    }

    fn get_address() -> Vec<String> {
        let result = std::env::var("ADDRESS")
            .map_err(|e| e.to_string())
            .and_then(|addrs| {
                Ok(addrs
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>())
            })
            .ok();
        let address = match result {
            Some(addrs) => addrs,
            None => {
                error!("无法从环境变量中获取ADDRESSES");
                Vec::new()
            }
        };
        address
    }

    pub async fn dispatch(&mut self) {
        self.logs.iter_log_files().await;

        // 日志分析
        for log in self.logs.iter_mut() {
            if !log.spawn && log.filename.exists() {
                log.spawn = true;
                tokio::spawn(read_log_file(log.clone()));
            }
        }
    }

    pub async fn upload(&self,body:LogChannel) {
        info!("上报数据:\n");
        let api = format!("{}/{}/{}",Monitor::get_config().await.api_report_log,self.server_id.unwrap_or_default(),body.filename);
        let client = ClientBuilder::new().build().unwrap();
        let result = client
            .post(api)
            .body(body.body)
            .send()
            .await;
        if result.is_err() {
            error!("上报数据失败:{:?}", result);
        }
    }

    pub async fn mining(&self, address: &str) -> Result<bool, String> {
        //测试地址
        let dir = std::env::current_dir().unwrap().join("run.sh");
        let result = std::process::Command::new("bash")
            .args([dir.to_str().unwrap(), &address])
            .output();
        match result {
            Ok(output) => {
                if output.status.success() {
                    Ok(true)
                } else {
                    Err(format!("运行退出异常：{:?}", output.status.code()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_config()->crate::config::Monitor{
        let config = Arc::clone(&CONFIG);
        let config_locked = config.lock().await;
        (*config_locked).monitor.clone()
    }
}

pub async fn monitor() {
    loop {
        let monitor = Arc::clone(&MONITOR);
        let mut monitor_locked = monitor.lock().await;
        // info!("{:?}", *monitor_locked);
        let reader = Arc::clone(&LOG);
        let mut reader_locked = reader.lock().await;

        tokio::select! {
            _ = (*monitor_locked).dispatch() => {
                // info!("{:?}",result);
                
            },
            Some(log_channel) =  (*reader_locked).1.recv() => {
                monitor_locked.upload(log_channel).await;
            }
        }
        drop(reader_locked);
        drop(monitor_locked);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
