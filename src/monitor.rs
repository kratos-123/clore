use indexmap::IndexMap;
use regex::Regex;
use reqwest::ClientBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tracing::error;
use tracing::info;

use self::nvidia::GeForces;
pub mod nvidia;
#[derive(Debug, Clone)]
struct Log {
    filename: PathBuf,
    spawn: bool,
}

pub async fn mining(address: &str) -> Result<bool, String> {
    //测试地址
    let dir = std::env::current_dir().unwrap().join("run.sh");
    let result = Command::new("bash")
        .args([dir.to_str().unwrap(), &address])
        .output()
        .await;
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

pub async fn get_task() {}

///! 日志收集和上报
pub async fn log_collect() {
    let geforces = GeForces::new();
    info!("geforces:{:?}",geforces);
    let mut paths = HashMap::<String, Log>::new();
    loop {
        collect_files(&mut paths).await;
        for (address, log) in paths.iter_mut() {
            if !log.spawn {
                log.spawn = true;
                info!("{:?}", log);
                let clone = log.clone();
                let addr = address.clone();
                tokio::spawn(async move { read_log_file(addr.replace(".txt", ""), clone).await });
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn collect_files(paths: &mut HashMap<String, Log>) {
    let path = std::env::current_dir().unwrap().join("log");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    let result = path.read_dir();
    for entry in result.unwrap() {
        if entry.is_ok() {
            let entry = entry.unwrap();
            let path: PathBuf = entry.path();
            if let Ok(metadata) = fs::metadata(&path).await {
                if metadata.is_file() {
                    let key = path.file_name().unwrap().to_str().unwrap().to_string();
                    let value = Log {
                        filename: path,
                        spawn: false,
                    };
                    if !paths.contains_key(&key) {
                        paths.insert(key, value);
                    }
                }
            }
        }
    }
}

async fn read_log_file(address: String, log: Log) {
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
                    request(&data.join("\n")).await;
                    hashstring.clear();
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn request(body: &str) {
    info!("上报数据:\n{}", body);
    // let client = ClientBuilder::new().build().unwrap();
    // let result = client
    //     .post(LOG_COLLECT_API)
    //     .body(body.to_string())
    //     .send()
    //     .await;
    // if result.is_err() {
    //     error!("上报数据失败:{:?}", result);
    // } else {
    //     info!("上报成功:{:?}", result)
    // }
}
