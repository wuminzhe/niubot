use chrono::Utc;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use rustc_serialize::hex::ToHex;
type HmacSha256 = Hmac<Sha256>;

pub fn sign(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_varkey(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.input(payload.as_bytes());
    let result = mac.result();
    result.code().to_hex()
}



pub fn get_unix_timestamp() -> i64 {
    let now = Utc::now();
    now.timestamp()
}

