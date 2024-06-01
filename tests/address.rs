pub mod common;
#[cfg(test)]
mod wallet {
    use std::{any::Any, collections::HashMap, sync::Arc};

    use monitor::server::address::{Address, AddressType, Wallet, WALLETS_STATE};
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
