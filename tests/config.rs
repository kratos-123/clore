pub mod common;

#[cfg(test)]
mod test {
    use monitor::config::Config;
    use std::any::{self, Any};

    use crate::common;

    #[tokio::test]
    async fn import_config_test() {
        common::setup();
        let result = Config::import_config();
        assert_eq!(
            any::TypeId::of::<Result<Config, String>>(),
            result.type_id()
        );
    }

    #[tokio::test]
    async fn export_config_test() {
        common::setup();
        let result = Config::import_config();
        assert_eq!(true, result.is_ok());
        let config = result.unwrap();
        assert_eq!(any::TypeId::of::<Config>(), config.type_id());
        let result = Config::export_config(&config);

        assert!(result.is_ok());
    }
}
