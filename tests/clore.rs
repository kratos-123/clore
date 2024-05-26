pub mod common;

#[cfg(test)]
mod test {

    use std::{any::Any, io::Read};

    use monitor::server::clore::{
        model::{market::Marketplace, Card},
        Clore,
    };
    use tracing::info;

    #[tokio::test]
    async fn marketplace_test() {
        crate::common::setup();
        let result = Clore::default().marketplace().await;
        info!("{:?}", result);
        assert_eq!(true, result.is_ok());
        if let Ok(cards) = result {
            let server_ids = cards
                .iter()
                .filter(|item| item.card_number == 2)
                .map(|item| item.server_id)
                .collect::<Vec<u32>>();
            info!("server_ids:{:?}", server_ids);
            if server_ids.len() > 0 {
                let resent_server_id = server_ids.get(0).unwrap();
                info!("resent_server_id:{:?}", server_ids);
            }
        }
    }

    #[tokio::test]
    async fn wallet_test() {
        crate::common::setup();
        let result = Clore::default().wallet().await;
        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn marketplace_json_test() {
        crate::common::setup();
        let row = String::from(
            r#"{"servers":[{"id":1398,"owner":979,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 97.5735GB","disk_speed":1754.3859649122808,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":2.99,"down":4.78,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":1,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":2},"cuda_version":"12.0"},{"id":1397,"owner":979,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 97.8746GB","disk_speed":1779.359430604982,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":3.64,"down":1.71,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":1,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":4},"cuda_version":"12.0"},{"id":1393,"owner":929,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 98.018GB","disk_speed":1742.1602787456447,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":0.24,"down":3.32,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":0.9999,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":2},"cuda_version":"12.0"},{"id":1229,"owner":775,"mrl":1440,"price":{"on_demand":{"bitcoin":0.0025},"spot":{"bitcoin":0.00017698}},"rented":false,"specs":{"mb":"ROG STRIX Z490-E GAMING","cpu":"Intel(R) Celeron(R) G5905 CPU @ 3.50GHz","cpus":"2/2","ram":15.8085,"disk":" 62.9768GB","disk_speed":2252.252252252252,"gpu":"4x NVIDIA GeForce RTX 3090","gpuram":24,"net":{"up":59.81,"down":39.86,"cc":"KR"},"backend_version":8,"pcie_rev":1,"pcie_width":1,"pl":[350,350,350,350]},"reliability":1,"allowed_coins":["bitcoin"],"rating":{"avg":3.5,"cnt":6},"cuda_version":"12.0"}],"my_servers":[],"code":0}"#,
        );
        let result = serde_json::from_str::<Marketplace>(&row);
        assert_eq!(true, result.is_ok());
    }

    #[tokio::test]
    async fn marketplace_filter_test() {
        crate::common::setup();
        let mut marketplace =
            std::fs::File::open("./market.json").expect("当前目录下./market.json文件不存在！");
        let mut row = String::from("");
        let _ = marketplace.read_to_string(&mut row);
        let result = serde_json::from_str::<Marketplace>(&row);
        assert_eq!(true, result.is_ok());
        let model = result.unwrap();
        let cards: Vec<Card> = model.filter();
        let server_ids = cards
            .iter()
            .filter(|item| item.card_number == 2)
            .map(|item| item.server_id)
            .collect::<Vec<u32>>();
        info!("server_ids:{:?}", server_ids);
        if server_ids.len() > 0 {
            let resent_server_id = server_ids.get(0).unwrap();
            info!("resent_server_id:{:?}", resent_server_id);
            let result = Clore::default()
                .create_order(resent_server_id.clone())
                .await;
            assert_eq!(true, result.is_ok())
        }

        assert_eq!(std::any::TypeId::of::<Vec<Card>>(), cards.type_id())
    }

    #[tokio::test]
    async fn create_order_test() {
        crate::common::setup();
        let result = Clore::default().create_order(77777).await;
        return;
        let market = Clore::default().marketplace().await;
        if let Ok(cards) = market {
            let server_ids = cards
                .iter()
                .filter(|item| item.card_number == 2)
                .map(|item| item.server_id)
                .collect::<Vec<u32>>();
            info!("server_ids:{:?}", server_ids);
            if server_ids.len() > 0 {
                let resent_server_id = server_ids.get(0).unwrap();
                info!("resent_server_id:{:?}", resent_server_id);
                let result = Clore::default()
                    .create_order(resent_server_id.clone())
                    .await;
                info!("create_order_test:{:?}", result);
                assert_eq!(true, result.is_ok())
            }
        }
    }

    #[tokio::test]
    async fn my_orders_test() {
        crate::common::setup();
        Clore::default().my_orders().await;
    }
}
