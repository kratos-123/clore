use monitor::monitor::log_collect;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    zipscreen();
    log_collect().await;
    Ok(())
}

fn zipscreen() {
    let _ = std::process::Command::new("screen")
        .args(["-d", "nimble"])
        .output();
}
