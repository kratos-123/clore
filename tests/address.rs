pub mod common;
#[cfg(test)]
mod wallet {
    use std::any::Any;

    use monitor::server::address::{Address, Wallet};
    use tracing::info;

    #[tokio::test]
    async fn load_address_file_test() {
        crate::common::setup();
        let instance = Address::default();
        let address = instance.load_address_file().await;
        info!("{:?}", address);
        assert_eq!(std::any::TypeId::of::<Vec<Wallet>>(), address.type_id())
    }
}
