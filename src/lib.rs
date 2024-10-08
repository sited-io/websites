use chrono::{DateTime, Utc};
use tonic::Status;

pub mod api;
mod auth;
pub mod cloudflare;
pub mod db;
pub mod images;
pub mod logging;
mod model;
pub mod publisher;
mod services;
pub mod zitadel;

pub use auth::init_jwks_verifier;
pub use services::*;

pub fn get_env_var(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| {
        panic!("ERROR: Missing environment variable '{var}'")
    })
}

pub fn datetime_to_timestamp(datetime: DateTime<Utc>) -> u64 {
    u64::try_from(datetime.timestamp()).unwrap()
}

pub fn i64_to_u32(n: i64) -> Result<u32, Status> {
    n.try_into().map_err(|err| {
        tracing::log::error!("{:?}", err);
        Status::internal("")
    })
}
