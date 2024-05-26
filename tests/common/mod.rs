use tracing::info;

pub fn setup() {
    let loginit = tracing_subscriber::fmt::try_init();
    if let Ok(()) = loginit {
        info!("日志初始化完成")
    }
}
