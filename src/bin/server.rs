use actix_web::{App, HttpServer};
use monitor::server::address::pool;
use monitor::server::distribute_address;
use monitor::server::printlnlog;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let mut tasks: Vec<_> = Vec::new();
    tasks.push(tokio::spawn(pool()));

    let _ = HttpServer::new(|| App::new().service(distribute_address).service(printlnlog))
        .bind(("0.0.0.0", 8888))?
        .run()
        .await;

    for task in tasks.into_iter() {
        task.abort();
    }

    Ok(())
}
