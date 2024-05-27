pub mod common;
#[cfg(test)]
mod test {
    use tracing::info;

    #[tokio::test]
    async fn nvidia_test() {
        crate::common::setup();
    }
}
