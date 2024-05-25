use actix_web::{App, HttpServer};
use monitor::server::distribute_address;
use monitor::server::printlnlog;
use monitor::server::wallet::pool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let mut tasks: Vec<_> = Vec::new();
    tasks.push(tokio::spawn(pool()));

    let _ = HttpServer::new(|| App::new().service(distribute_address).service(printlnlog))
        .bind(("127.0.0.1", 8888))?
        .run()
        .await;

    for task in tasks.into_iter() {
        task.abort();
    }

    Ok(())
}
