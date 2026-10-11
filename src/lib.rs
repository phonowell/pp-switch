pub mod client;
pub mod config;
pub mod provider;
pub mod storage;
pub mod ui;

pub type Result<T> = std::result::Result<T, String>;
