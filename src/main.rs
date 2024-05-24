use actix_web::{web, App, HttpResponse, HttpServer};
use clore::{
    clore::log_collect,
    server::{mining, printlnlog},
    wallet::pool,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    zipscreen();
    tracing_subscriber::fmt::init();
    let mut tasks = vec![];
    tasks.push(tokio::spawn(log_collect()));
    tasks.push(tokio::spawn(pool()));
    let _ = HttpServer::new(|| App::new().service(mining).service(printlnlog))
        .bind(("127.0.0.1", 8888))?
        .run()
        .await;
    for task in tasks.into_iter() {
        task.abort();
    }
    // let _ = clore::clore::Clore::default().marketplace().await;
    Ok(())
}

fn zipscreen() {
    let _ = std::process::Command::new("screen")
        .args(["-d", "nimble"])
        .output();
}
