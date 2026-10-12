pub mod client;
pub mod config;
pub mod i18n;
pub mod protocol;
pub mod provider;
pub mod storage;
mod theme;
pub mod ui;
pub mod update;

pub type Result<T> = std::result::Result<T, String>;
