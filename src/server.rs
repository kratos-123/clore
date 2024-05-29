use actix_web::{get, post};
use tracing::error;
use tracing::{info, warn};

pub mod clore;
pub mod ssh;
pub mod wallet;

#[get("/distribute_address/{card_number}/{server_id}")]
pub async fn distribute_address() -> String {
    unimplemented!()
}

#[post("/printlnlog")]
pub async fn printlnlog(body: String) -> String {
    let regex = regex::Regex::new(r"err|Err").unwrap();
    if regex.is_match(&body) {
        error!("{:?}", body);
    } else {
        info!("{:?}", body);
    }

    "ok".to_string()
}
