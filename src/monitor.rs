
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::process::Command;
use tracing::error;
use tracing::warn;

use crate::clore::LOG_COLLECT_API;

pub async fn mining(address:&str) -> Result<bool,String> {
    //测试地址
    let dir = std::env::current_dir().unwrap().join("run.sh");
    let result = Command::new("bash")
        .args([
            dir.to_str().unwrap(),
            &address,
            ">>",
            "log.txt",
            "2>&1",
            "&",
        ])
        .output().await;
    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            }else{
                Err(format!("运行退出异常：{:?}",output.status.code()))
            }
        },
        Err(e) => {
            Err(e.to_string())
        },
    }
}


///! 日志收集和上报
pub async fn log_collect() {
    let path = std::env::current_dir().unwrap().join("log.txt");
    loop {
        if path.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let command = Command::new("tail")
        .args(["-f", &path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .spawn();
    let client = reqwest::ClientBuilder::new().build().unwrap();
    match command {
        Ok(child) => {
            let stdout = child.stdout.unwrap();
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let uploade = client.post(LOG_COLLECT_API).body(line).send().await;
                if let Err(e) = uploade {
                    warn!("LOG_COLLECT_API:{:?}", e.to_string());
                }
            }
        },
        Err(e) => {
            error!("不支持当前系统环境：{:?}",e)
        },
    }
}
