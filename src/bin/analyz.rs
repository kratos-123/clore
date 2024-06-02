use monitor::log::Logs;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    Logs::monitor().await;
}
