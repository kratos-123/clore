use actix_web::{web, App, HttpResponse, HttpServer};
use clore::wallet::pool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    // HttpServer::new(|| App::new().route("/", web::get().to(HttpResponse::Ok)))
    //     .bind(("127.0.0.1", 8888))?
    //     .run()
    //     .await
    pool().await;
    Ok(())
}
