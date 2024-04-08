use crate::utils::compute_random_32_bytes_key;
use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub argon_secret: String,
    pub access_token_secret: String,
    pub refresh_token_secret: String,
    pub refresh_token_ttl_sec: u64,
    pub access_token_ttl_sec: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            argon_secret: compute_random_32_bytes_key(),
            access_token_secret: compute_random_32_bytes_key(),
            refresh_token_secret: compute_random_32_bytes_key(),
            refresh_token_ttl_sec: 172800,
            access_token_ttl_sec: 3600,
        }
    }
}
