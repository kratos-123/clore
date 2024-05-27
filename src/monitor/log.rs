use core::ascii;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, path::PathBuf, process::Command};

use indexmap::IndexMap;
use regex::Regex;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tracing::info;

#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    pub filename: PathBuf,
    pub spawn: bool,
}

#[derive(Debug)]
pub struct Logs(Vec<Log>);

impl Logs {
    pub fn new() -> Logs {
        Logs(Vec::new())
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
    pub async fn iter_log_files(&mut self)
    {
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
                        };
                        if !(*self).contains(&value) {
                            (*self).push(value);
                        }
                    }
                }
            }
        }
    }
}

pub async fn read_log_file(log: Log) {
    let address = log.filename.file_name().unwrap_or_default().to_str().unwrap().to_string();
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
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut hashstring = IndexMap::<String, String>::new();
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
                    let data = hashstring
                        .iter()
                        .map(|(_, item)| item.clone())
                        .collect::<Vec<String>>();
                    // request(&data.join("\n")).await;
                    hashstring.clear();
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        }
    }
}
