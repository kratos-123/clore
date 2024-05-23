use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tracing::error;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::process::Command;
use tracing::{info, warn};
const API:&str = "http://127.0.0.1:8888/printlnlog";

#[get("/mining/{addrss}")]
pub async fn mining(path: web::Path<(String)>) -> impl Responder {
    //测试地址
    let (address) = path.into_inner();
    let dir = std::env::current_dir().unwrap().join("run.sh");
    let result = Command::new("bash")
        .args([dir.to_str().unwrap(), &address])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string());
    let clone_addr = address.clone();
    if let Ok(command) = result {
        let mut stdout = command.stdout.unwrap();
        tokio::spawn(async move {
            let reader = BufReader::new(&mut stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.is_empty() {
                    let body = format!("{} {}",clone_addr,line.clone());
                    let client = reqwest::Client::new();
                    let _ = client
                        .post(API)
                        .body(body)
                        .send()
                        .await;
                    let file = tokio::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open("./log.txt")
                        .await
                        .unwrap();

                    let mut writer = BufWriter::new(file);
                    let _ = writer.write_all(&line.as_bytes());
                }
            }
        });
    }
    HttpResponse::Ok().body(dir.to_str().unwrap().to_owned())
}

#[post("/printlnlog")]
pub async fn printlnlog(body: String) -> String {
    let regex = regex::Regex::new(r"err|Err").unwrap();
    if regex.is_match(&body) {
        error!("{:?}", body);
    }else {
        info!("{:?}", body);
    }

    "ok".to_string()
}
