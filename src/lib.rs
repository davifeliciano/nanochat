pub mod auth;
pub mod chat;
pub mod config;
pub mod db;
pub mod users;
pub mod utils;

pub trait Validate {
    fn validate(&self) -> bool;
}
