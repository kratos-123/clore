pub mod common;
#[cfg(test)]
mod wallet {
    use std::{any::Any, collections::HashMap, sync::Arc};

    use monitor::server::wallet::{AddressType, Wallet, Wallets, WALLETS_STATE};
    use tracing::info;

    #[tokio::test]
    async fn load_address_file_test() {
        crate::common::setup();
        let address = Wallets::load_address_file().await;
        info!("{:?}", address);
        assert_eq!(std::any::TypeId::of::<Vec<Wallet>>(), address.type_id())
    }

    #[tokio::test]
    async fn check_test() {
        crate::common::setup();
        // 主通过    nimble1fc7l9qmgm3q42yuc7qpy3yed83xk9wjqy8vw0u
        // 子通过    nimble1quz2sl26h8n7rg48juc6xalekhxp0dle3k8f2e
        // 未通过    nimble1enex83alluyduwwg85fvqhdadnkyflu2x6mpcg
        let mut other = HashMap::<String, Wallet>::new();
        let wallets = Arc::clone(&WALLETS_STATE);
        let mut row = wallets.lock().await;
        let mut address = "nimble1fc7l9qmgm3q42yuc7qpy3yed83xk9wjqy8vw0u";

        // 主地址匹配
        let master = vec![Wallet::new(address.to_string(), AddressType::NULL)];
        other.clear();
        other.insert(
            address.to_string(),
            Wallet::new(address.to_string(), AddressType::MASTER),
        );
        row.clear();
        row.check(&master).await;
        assert_eq!((*row).0, other);

        // 子地址测试
        address = "nimble1quz2sl26h8n7rg48juc6xalekhxp0dle3k8f2e";
        let sub = vec![Wallet::new(address.to_string(), AddressType::NULL)];
        other.clear();
        other.insert(
            address.to_string(),
            Wallet::new(address.to_string(), AddressType::SUB),
        );
        row.clear();
        row.check(&sub).await;
        assert_eq!((*row).0, other);

        // 未审核通过测试
        address = "nimble1enex83alluyduwwg85fvqhdadnkyflu2x6mpcg";
        let unregister = vec![Wallet::new(address.to_string(), AddressType::NULL)];
        other.clear();
        other.insert(
            address.to_string(),
            Wallet::new(address.to_string(), AddressType::NULL),
        );
        row.clear();
        row.check(&unregister).await;
        assert_ne!((*row).0, other);
    }
}
