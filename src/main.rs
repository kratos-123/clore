use monitor::monitor::monitor;
use time::{macros::format_description, UtcOffset};
use tracing_subscriber::{fmt::time::OffsetTime};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"),
    );
    tracing_subscriber::fmt().with_timer(local_time).init();
    zipscreen();
    monitor().await;
    Ok(())
}

fn zipscreen() {
    let _ = std::process::Command::new("screen")
        .args(["-d", "nimble"])
        .output();
}
