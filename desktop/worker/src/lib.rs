use serde::{Serialize, Deserialize};
use gloo_worker::oneshot;
use wwise_format::Soundbank;
use wwise_format::parse_soundbank;

#[oneshot::oneshot]
pub async fn ParseWorker(
    bytes: Vec<u8>,
) -> Result<Soundbank, String> {
    parse_soundbank(&bytes)
        .map_err(|_| "Could not parse BNK file".to_string())
}
