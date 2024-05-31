use lazy_static::lazy_static;
use pm::Process;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};
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
pub mod pm;
lazy_static! {
    pub static ref MONITOR: Arc<Mutex<Monitor>> = Arc::new(Mutex::new(Monitor::new()));
    pub static ref LOG: Arc<Mutex<(UnboundedSender<LogChannel>, UnboundedReceiver<LogChannel>)>> =
        Arc::new(Mutex::new(unbounded_channel::<LogChannel>()));
}

#[derive(Debug, Clone)]
pub struct LogChannel {
    filename: String,
    body: String,
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
                error!("无法从环境变量中获取ADDRESS");
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
            error!("无法从环境变量中获取当前card_number")
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

        info!("获取运行命令输出:{}\n", row);
        let reg: regex::Regex = regex::Regex::new(r"(nimble[\w]+)").map_err(|e| e.to_string())?;
        for command in row.split("\n") {
            if !command.is_empty() {
                info!("{:?}", command);
                let (_, [addr]) = reg.captures(command).ok_or("无匹配值！")?.extract::<1>();
                address.push(addr.to_string());
            }
        }
        info!("后台运行地址:{:?}", address);

        Ok(address)
    }

    // 本地监控
    pub async fn mining(&self) -> Result<(), String> {
        let address = self.address.clone();
        let nvidias = self.nvidias.clone();
        let process = self.py_pros().await?;

        // 正常运行
        if process.len() == address.len() && process.len() == (nvidias).len() {
            info!("服务运行正常");
            return Ok(());
        }
        let result = Process::new();
        if let Err(e) = result {
            let e = format!("初始化pm2命令失败:{}", e);
            error!(e);
            return Err(e);
        }
        let pm2 = result.unwrap();
        for (index, addr) in address.iter().enumerate() {
            let action_name = format!("nimble{}", index);
            let action = pm2.get_action(&action_name);
            let dir = std::env::current_dir().unwrap().join("execute.sh");
            let mut bash = std::process::Command::new("bash");
            match action {
                pm::Action::START => {
                    info!("创建挖矿程序中...");
                    bash.args([
                        dir.to_str().unwrap(),
                        "start",
                        index.to_string().as_str(),
                        &addr,
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

            if let Err(e) = bash.output() {
                error!("重启挖矿程序失败:{}", e.to_string());
            }else{
                info!("已重启挖矿程序");
            }
        }
        Ok(())
    }

    pub async fn dispatch(&mut self) {
        // 日志分析
        self.logs.iter_log_files().await;
        for log in self.logs.iter_mut() {
            if !log.spawn && log.filename.exists() {
                log.spawn = true;
                tokio::spawn(read_log_file(log.clone()));
            }
        }

        //监控是否掉线

        let result = self.mining().await;
        if let Err(e) = result {
            error!("调用程序失败:{}", e);
        }
    }

    pub async fn upload(&self, body: LogChannel) {
        if body.body.is_empty() {
            return;
        }
        let api = format!(
            "{}/{}/{}",
            Monitor::get_config().await.api_report_log,
            self.server_id.unwrap_or_default(),
            body.filename
        );
        info!("上报数据:\n{}\n\n{}", api, body.body);
        let client = ClientBuilder::new().build().unwrap();
        let result = client.post(api).body(body.body).send().await;
        if result.is_err() {
            error!("上报数据失败:{:?}", result);
        }
    }

    pub async fn get_config() -> crate::config::Monitor {
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
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
