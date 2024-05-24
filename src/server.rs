use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::process::Command;
use tracing::error;
use tracing::{info, warn};

#[get("/mining/{addrss}")]
pub async fn mining(path: web::Path<(String)>) -> impl Responder {
    //测试地址
    let (address) = path.into_inner();
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
        .spawn()
        .map_err(|e| e.to_string());
    match result {
        Ok(_) => "ok".to_string(),
        Err(e) => e,
    }
}

#[post("/printlnlog")]
pub async fn printlnlog(body: String) -> String {
    let regex = regex::Regex::new(r"err|Err").unwrap();
    if regex.is_match(&body) {
        error!("{:?}", body);
    } else {
        info!("{:?}", body);
    }

    "ok".to_string()
}
