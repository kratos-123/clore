use std::fs::OpenOptions;
use std::io::Write;

use actix_web::web;
use actix_web::{get, post};
use tracing::error;
use tracing::info;

pub mod address;
pub mod clore;
pub mod ssh;

#[get("/distribute_address/{card_number}/{server_id}")]
pub async fn distribute_address() -> String {
    unimplemented!()
}

#[post("/printlnlog/{server_id}/{filename}")]
pub async fn printlnlog(body: String, pathinfo: web::Path<(String, String)>) -> String {
    let regex = regex::Regex::new(r"err|Err").unwrap();
    if regex.is_match(&body) {
        error!("{:?}", body);
    } else {
        info!("{:?}", body);
    }
    let (server_id, filename) = pathinfo.into_inner();
    if let Ok(filepath) = std::env::current_dir() {
        let filename = format!("{}{}.log", filename, server_id);
        let mut path = filepath.join("logs");
        if !path.exists() {
            let _ = std::fs::create_dir_all(path.clone());
        }
        path = path.join(filename);
        let isopened = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(path.clone());
        if let Ok(writer) =  isopened {
            let mut buf = std::io::BufWriter::new(writer);
            let row = format!("{}\n",body);
            let _ = buf.write_all(row.as_bytes());
        }else {
            error!("打开文件:{} 写入失败!",path.as_path().display());
        }

    }

    "ok".to_string()
}
