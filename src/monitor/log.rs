use core::ascii;
use std::fs;
use std::io::SeekFrom;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf, process::Command};

use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt};
use tokio::io::BufReader;
use tracing::{error, info};

use super::LOG;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadType {
    LINES,
    ALL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub filename: PathBuf,
    pub spawn: bool,
    pub read_type:ReadType
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
       let mut logs =  Logs(Vec::new());
       let log_path= std::env::current_dir().unwrap().join("nimble-miner-public/my_logs.json");
       logs.add_log_file(log_path,ReadType::ALL);
       logs
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
        let path = std::env::current_dir().unwrap().join("log");
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
                            read_type:ReadType::LINES
                        };
                        if !(*self).contains(&value) {
                            (*self).push(value);
                        }
                    }
                }
            }
        }
    }

    pub fn add_log_file(&mut self,log_path:PathBuf,read_type:ReadType){
        let log: Log = Log{ filename: log_path, spawn: false ,read_type};
        self.push(log)
    }
}

pub async fn read_log_file(log: Log) {
    info!("监听新文件日志:{:?}", log.filename);
    let address = log
        .filename
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap()
        .to_string()
        .replace(".txt", "");
    let result = tokio::fs::OpenOptions::new()
        .read(true)
        .open(log.filename.clone())
        .await;
    if let Ok(file) = result {
        // 任务下载
        // (Downloading)([\w ]*:)*[\t ]+([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+
        // 生成任务和任务测试
        // (Generating)([\w ]*:)*[\t ]+([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+
        // 映射任务
        // (Map)([\w ]*:)?[\t ]+([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+
        // 三合一规则
        // (Generating|Downloading|Map)([\w ]*:)[\t ]+([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+

        // 算力测试
        // ([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+

        let complex_regex = Regex::new(
            r"(Generating|Downloading|Map)([\w ]*:)[\t ]+([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+",
        )
        .unwrap();
        let bit_reg =
            Regex::new(r"([\d]+%)\|[\S ]+\|[ ]+(\S+)[ +][\[\S]+[ ]+([\d\.\w<?]+)[ \w\/\]]+")
                .unwrap();
        let mut reader = BufReader::new(file);
        let mut hashstring = IndexMap::<String, String>::new();
        match log.read_type {
            ReadType::LINES => {
                let mut lines = reader.lines();
                while let Ok(some_line) = lines.next_line().await {
                    if let Some(line) = some_line {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        // 复合正则规则过滤进度条
                        let result = complex_regex.captures(&line);
                        if result.is_some() {
                            let captures = result.unwrap();
                            let (_, [operate, extra, percent, task, downspeed]) = captures.extract();
                            let string = format!(
                                "{} {}{} {} {} {}",
                                address, operate, extra, percent, task, downspeed
                            );
                            hashstring.insert(format!("{}{}", operate, extra), string);
                        } else {
                            let result = bit_reg.captures(&line);
                            if result.is_some() {
                                let captures = result.unwrap();
        
                                let (_, [percent, task, downspeed]) = captures.extract();
                                let string = format!("{} {} {} {}", address, percent, task, downspeed);
                                hashstring.insert("task_prcess".to_string(), string);
                            } else {
                                let string = format!("{} {}", address, line);
                                hashstring.insert(line.to_string(), string);
                            }
                        }
                    } else {
                        if !hashstring.is_empty() {
                            let message = hashstring
                                .iter()
                                .map(|(_, item)| item.clone())
                                .collect::<Vec<String>>()
                                .join("\n");
                            let reader = Arc::clone(&LOG);
                            let reader_locked = reader.lock().await;
                            let result = reader_locked.0.send(message);
                            if result.is_err() {
                                error!("日志上传失败:{:?}", result);
                            }
            
                            drop(reader_locked);
                            hashstring.clear();
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            },
            ReadType::ALL => {
                loop {
                    let mut buf = String::from("");
                    let _ = reader.read_to_string(&mut buf).await;
                    info!(buf);
                    if !buf.is_empty() {
                        buf = buf.replace(" ", "").replace("\n", "").replace("\r", "");
                        let reader_chan = Arc::clone(&LOG);
                        let reader_chan_locked = reader_chan.lock().await;
                        let result = reader_chan_locked.0.send(buf.clone());
                        if result.is_err() {
                            error!("日志上传失败:{:?}", result);
                        }else {
                            info!("监听系统自带运行日志，成功上传");
                        }
                        let _ = reader.seek(SeekFrom::Start(0)).await;
                        drop(reader_chan_locked);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            },
        }
        
    }
}
