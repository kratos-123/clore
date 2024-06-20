use actix_web::{App, HttpServer};
use monitor::server::address::pool;
use monitor::server::distribute_address;
use monitor::server::printlnlog;
use time::macros::format_description;
use time::UtcOffset;
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"),
    );
    tracing_subscriber::fmt().with_timer(local_time).init();

    pool().await;
    // let mut tasks: Vec<_> = Vec::new();
    // tasks.push(tokio::spawn(pool()));

    // let _ = HttpServer::new(|| App::new().service(distribute_address).service(printlnlog))
    //     .bind(("0.0.0.0", 8888))?
    //     .run()
    //     .await;

    // for task in tasks.into_iter() {
    //     task.abort();
    // }

    Ok(())
}
