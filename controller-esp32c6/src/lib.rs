#![no_std]
#![feature(const_trait_impl)]
#![feature(const_index)]

extern crate alloc;

pub mod backend;
pub mod config;
mod form;
mod http;
mod portal;
pub mod sensor;

pub use portal::{ConfigPortalError, connect_wifi, run_config_portal};

pub fn init_logging() {
    esp_println::logger::init_logger(log::LevelFilter::Debug);
    log::debug!("Debug logging enabled");
}
