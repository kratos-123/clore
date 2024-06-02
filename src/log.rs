use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{error, info};

use crate::monitor::Monitor;

lazy_static! {
    pub static ref LOG_CHANNEL: Arc<Mutex<(UnboundedSender<LogChannel>, UnboundedReceiver<LogChannel>)>> =
        Arc::new(Mutex::new(unbounded_channel::<LogChannel>()));
    pub static ref LOG_FILES: Arc<Mutex<Logs>> = Arc::new(Mutex::new(Logs::new()));
}

#[derive(Debug, Clone)]
pub struct LogChannel {
    filename: String,
    body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub filename: PathBuf,
    pub spawn: bool,
}

impl PartialEq for Log {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logs(Vec<Log>);

impl Logs {
    pub fn new() -> Logs {
        let mut logs = Logs(Vec::new());
        let log_path = std::env::current_dir()
            .unwrap()
            .join("nimble-miner-public/my_logs.json");
        logs.add_log_file(log_path);
        logs
    }

    pub async fn upload(body: LogChannel) {
        if body.body.is_empty() {
            return;
        }
        let api = format!(
            "{}/{}/{}",
            Monitor::get_config().await.api_report_log,
            Monitor::get_server_id().unwrap_or_default(),
            body.filename
        );
        info!("上报数据:\n{}\n\n{}", api, body.body);
        let client = ClientBuilder::new().build().unwrap();
        let result = client.post(api).body(body.body).send().await;
        if result.is_err() {
            error!("上报数据失败:{:?}", result);
        }
    }
}

impl Deref for Logs {
    type Target = Vec<Log>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Logs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Logs {
    pub async fn iter_log_files(&mut self) {
        let path = std::env::current_dir().unwrap().join("logs");
        if !path.exists() {
            let _ = std::fs::create_dir_all(&path);
        }
        let result = path.read_dir();
        for entry in result.unwrap() {
            if entry.is_ok() {
                let entry = entry.unwrap();
                let path: PathBuf = entry.path();
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_file() {
                        let value = Log {
                            filename: path,
                            spawn: false,
                        };
                        if !(*self).contains(&value) {
                            (*self).push(value);
                        }
                    }
                }
            }
        }
    }

    pub fn add_log_file(&mut self, log_path: PathBuf) {
        let log: Log = Log {
            filename: log_path,
            spawn: false,
        };
        self.push(log)
    }

    pub async fn read_log_file(log: Log) {
        info!("监听新文件日志:{:?}\n", log.filename);
        let address = log
            .filename
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap()
            .to_string()
            .replace(".txt", "")
            .replace(".json", "");
        let result = tokio::fs::OpenOptions::new()
            .read(true)
            .open(log.filename.clone())
            .await
            .map_err(|e| e.to_string());
        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
        let file = result.unwrap();
        let reader = BufReader::new(file);
        let mut hashstring = IndexMap::<String, String>::new();
        let complex_regex = Regex::new(
            r"(Generating|Downloading|Map)([\w ]*:)[\t ]+([\d\.]+\%)\|[\S ]+\|[ ]+([\d]+)\/([\d]+)[ +][\[\S]+[ ]+([\d\.]+)[\w<? \w\/\]]+",
        )
        .unwrap();

        let bit_reg = Regex::new(
            r"([\d]+%)\|[\S ]+\|[ ]+([\d]+)\/([\d]+)[ +][\[\S]+[ ]+([\d\.]+)[ \w\/\]<?]+",
        )
        .unwrap();

        let verify = Regex::new(r"\{'(loss|eval_loss).*}").unwrap();

        let complated = Regex::new(r"completed the task.*").unwrap();
        let mut instant = tokio::time::Instant::now();
        let mut lines = reader.lines();
        while let Ok(some_line) = lines.next_line().await {
            if let Some(line) = some_line {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if verify.captures(&line).is_some() {
                    continue;
                }
                let complex = complex_regex.captures(&line);
                if complex.is_some() {
                    let captures = complex.unwrap();
                    let (_, [operate, extra, percent, prce, total, downspeed]) =
                        captures.extract::<6>();
                    let string = format!(
                        "当前操作:{}{},完成百分比:{},完成进度:{}/{} 下载速度:{}",
                        operate, extra, percent, prce, total, downspeed
                    );
                    hashstring.insert(format!("{}{}", operate, extra), string);
                    continue;
                }

                let bittest = bit_reg.captures(&line);
                if bittest.is_some() {
                    let captures = bittest.unwrap();
                    let (_, [percent, prce, total, it]) = captures.extract::<4>();
                    let it = it.parse::<f32>().unwrap_or_default();
                    let string = format!("{} {} {} {}", percent, prce, total, it);
                    // 验算时，这个算力的值非常大，不应该算进到日志里面去
                    if it > 35f32 {
                        continue;
                    }

                    hashstring.insert("task_prcess".to_string(), string);
                    continue;
                }

                let string = format!("{} {}", address, line);
                hashstring.insert(line.to_string(), string);
            }

            if !hashstring.is_empty() && instant.elapsed() > tokio::time::Duration::from_secs(5) {
                let body = hashstring
                    .iter()
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<String>>()
                    .join("\n");
                let digest = format!("{:?}", md5::compute(body.as_bytes()));
                let split = "-".repeat(100);
                hashstring.insert(digest, split);

                println!("{}", body);
                instant = Instant::now();
                hashstring.clear();
                if complated.captures(&body).is_some() {
                    continue;
                }
     
            }
        }
    }

    pub async fn monitor() {
        loop {
            let log_files = Arc::clone(&LOG_FILES);
            let mut log_files_locked = log_files.lock().await;
            // 日志分析
            (*log_files_locked).iter_log_files().await;
            for log in (*log_files_locked).iter_mut() {
                if !log.spawn && log.filename.exists() {
                    log.spawn = true;
                    tokio::spawn(Logs::read_log_file(log.clone()));
                }
            }
            drop(log_files_locked);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum Status {
    Success,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunLog {
    #[serde(rename(deserialize = "WalletAddr"))]
    pub wallet_addr: String,
    #[serde(rename(deserialize = "CompletedTime"))]
    pub completed_time: String,
    #[serde(rename(deserialize = "TrainRuntime"))]
    pub trainrun_time: f64,
    #[serde(rename(deserialize = "Status"))]
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
struct RunLogs(Vec<RunLog>);

impl Deref for RunLogs {
    type Target = Vec<RunLog>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RunLogs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for RunLogs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for log in (*self).iter() {
            let s = format!(
                "{} {} {} {:0.5}\n",
                log.wallet_addr, log.status, log.completed_time, log.trainrun_time
            );
            let _ = f.write_str(&s);
        }

        Ok(())
    }
}
