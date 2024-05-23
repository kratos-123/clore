use actix_web::{web, App, HttpResponse, HttpServer};
use clore::{
    server::{mining, printlnlog},
    wallet::pool,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let _ = HttpServer::new(|| App::new().service(mining).service(printlnlog))
        .bind(("127.0.0.1", 8888))?
        .run()
        .await;
    // pool().await;
    // let _ = clore::clore::Clore::default().marketplace().await;
    Ok(())
}
