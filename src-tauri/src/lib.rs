mod commands;
mod db;
mod error;
mod oui;
mod parser;
mod protocols;

use std::sync::{Arc, Mutex};

pub use error::TaplootError;

pub struct AppState {
    pub db: Mutex<Option<Arc<Mutex<rusqlite::Connection>>>>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::import::import_pcap,
            commands::query::get_hosts,
            commands::query::get_connections,
            commands::query::get_time_range,
            commands::query::save_node_position,
            commands::query::get_host_detail,
            commands::query::get_connection_packets,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run taploot");
}
