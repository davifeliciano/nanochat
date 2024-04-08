pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}

pub mod config;
pub mod db;
pub mod utils;
