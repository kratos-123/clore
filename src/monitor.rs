use lazy_static::lazy_static;
use pm::{Action, Process};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use self::nvidia::GeForces;
use crate::config::CONFIG;
use crate::log::{self, LOG_CHANNEL};

pub mod nvidia;
pub mod pm;
lazy_static! {
    pub static ref MONITOR: Arc<Mutex<Monitor>> = Arc::new(Mutex::new(Monitor::new()));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monitor {
    server_id: Option<u32>,
    address: Vec<String>,
    nvidias: GeForces,
    upload_log: HashMap<String, Vec<String>>,
}

impl Monitor {
    fn new() -> Monitor {
        Monitor {
            server_id: Monitor::get_server_id(),
            address: Monitor::get_address(),
            nvidias: GeForces::new(),
            upload_log: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn get_server_id() -> Option<u32> {
        let server_id = std::env::var("SERVER_ID")
            .map_err(|e| e.to_string())
            .and_then(|server_id| server_id.parse::<u32>().map_err(|e| e.to_string()))
            .ok();
        if server_id.is_none() {
            error!("无法从环境变量中获取:SERVER_ID")
        }
        server_id
    }

    fn get_address() -> Vec<String> {
        let result = std::env::var("ADDRESS")
            .map_err(|e| e.to_string())
            .and_then(|addrs| {
                Ok(addrs
                    .split("-")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>())
            })
            .ok();
        let address = match result {
            Some(addrs) => addrs,
            None => {
                error!("无法从环境变量中获取:ADDRESS");
                Vec::new()
            }
        };
        address
    }

    pub async fn get_card_number() -> Option<u32> {
        let card_number = std::env::var("CARD_NUMBER")
            .map_err(|e| e.to_string())
            .and_then(|card_number| card_number.parse::<u32>().map_err(|e| e.to_string()))
            .ok();
        if card_number.is_none() {
            error!("无法从环境变量中获取:CARD_NUMBER")
        }
        card_number
    }

    async fn py_pros(&self) -> Result<Vec<String>, String> {
        info!("检测后台python挖矿程序");
        //ps -aeo command |grep execute.py |grep -v grep
        let mut address = Vec::new();
        let py_proc = Command::new("ps")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .args(["-aeo", "command"])
            .spawn()
            .map_err(|e| e.to_string())?
            .stdout
            .ok_or("ps命令运行失败！")?;
        let grep_py = Command::new("grep")
            .stdin(py_proc)
            .stdout(Stdio::piped())
            .args(["execute.py"])
            .spawn()
            .map_err(|e| e.to_string())?
            .stdout
            .ok_or("运行grep execute.py 失败！")?;
        let grep = Command::new("grep")
            .stdin(grep_py)
            .stdout(Stdio::piped())
            .args(["-v", "grep"])
            .output()
            .map_err(|e| e.to_string())?
            .stdout;
        let row = String::from_utf8(grep).map_err(|e| e.to_string())?;

        info!("检测挖矿挖矿程序,程序输出:\n{}", row);
        let reg: regex::Regex = regex::Regex::new(r"(nimble[\w]+)").map_err(|e| e.to_string())?;
        for command in row.split("\n") {
            if !command.is_empty() {
                info!("{:?}", command);
                let (_, [addr]) = reg.captures(command).ok_or("无匹配值！")?.extract::<1>();
                address.push(addr.to_string());
            }
        }
        if address.len() == 0 {
            let massge = format!("挖矿进程已退出！");
            error!("{}", massge);
            Err(massge)
        } else {
            info!(
                "后台运行地址数量:{}个,地址信息:{}",
                address.len(),
                address.join(",")
            );
            Ok(address)
        }
    }

    // 本地监控
    pub async fn mining(&self) -> Result<(), String> {
        let address = self.address.clone();
        if address.len() == 0 {
            let message = format!("无法从环境变量中获取地址信息，请检查您的环境变量");
            return Err(message);
        }
        let nvidias = self.nvidias.get_normal_nvidias();
        let py_pros = self.py_pros().await;
        if let Err(_) = &py_pros {
            error!("挖矿程序检测异常，正在进行拉起nimble服务");
        }
        let process = py_pros.unwrap_or_default();
        // 正常运行
        if process.len() == address.len() && process.len() == nvidias.len() {
            info!("服务正常!!");
            return Ok(());
        }
        let result = Process::new();
        if let Err(e) = result {
            let e = format!("pm2运行失败:{}", e);
            error!("{}", e);
            return Err(e);
        }
        let pm2 = result.unwrap();
        let mut action = Action::SKIP;
        for (index, addr) in address.iter().enumerate() {
            let action_name = format!("nimble{}", index);
            action = pm2.get_action(&action_name);
            let _ = self.pm2(action, index, addr.clone()).await;
        }
        Ok(())
    }

    async fn pm2(&self, action: pm::Action, index: usize, address: String) -> Result<(), String> {
        let dir = std::env::current_dir().unwrap().join("execute.sh");
        let mut bash = std::process::Command::new("bash");
        bash.stdout(Stdio::null()).stderr(Stdio::null());

        match action {
            pm::Action::START => {
                bash.args([
                    dir.to_str().unwrap(),
                    "start",
                    index.to_string().as_str(),
                    &address,
                ]);
            }
            pm::Action::RESTART => {
                info!("正在重启挖矿程序!");
                bash.args([dir.to_str().unwrap(), "restart", index.to_string().as_str()]);
            }
            pm::Action::SKIP => {
                bash.args(["echo", "'done'"]);
            }
        }
        let _ = bash.spawn().map_err(|e| e.to_string())?.wait();
        info!("已重新拉起挖矿程序！");
        Ok(())
    }

    pub async fn dispatch(&mut self) {
        //监控是否掉线

        let result = self.mining().await;
        if let Err(e) = result {
            error!("调用程序失败:{}", e);
        }
    }

    pub async fn get_config() -> crate::config::Monitor {
        let config = Arc::clone(&CONFIG);
        let config_locked = config.lock().await;
        (*config_locked).monitor.clone()
    }
}

pub async fn monitor() {
    // tokio::spawn(log::Logs::monitor());

    loop {
        let monitor = Arc::clone(&MONITOR);
        let mut monitor_locked = monitor.lock().await;
        let log_channel = Arc::clone(&LOG_CHANNEL);
        let mut log_channel_locked = log_channel.lock().await;

        tokio::select! {
            _ = (*monitor_locked).dispatch() => {

            },
            Some(msg) = (*log_channel_locked).1.recv() =>{
                match msg.msg_type {
                    log::MsgType::RESTART => {
                        warn!("需要重启:{:?}",msg);
                    },
                    _=>{}
                }
            }
        }

        // (*monitor_locked).dispatch().await;
        drop(monitor_locked);
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
