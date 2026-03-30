use serde::Serialize;
use tauri::State;

use crate::db::queries;
use crate::{AppState, TaplootError};

#[derive(Debug, Serialize)]
pub struct Host {
    pub id: i64,
    pub mac_address: String,
    pub ip_address: String,
    pub device_type: String,
    pub first_seen: f64,
    pub last_seen: f64,
}

#[derive(Debug, Serialize)]
pub struct Connection {
    pub id: i64,
    pub src_host_id: i64,
    pub dst_host_id: i64,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub app_protocol: Option<String>,
    pub packet_count: i64,
    pub byte_count: i64,
    pub first_seen: f64,
    pub last_seen: f64,
}

fn get_db(
    state: &State<'_, AppState>,
) -> Result<std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>, TaplootError> {
    let db_lock = state
        .db
        .lock()
        .map_err(|e| TaplootError::Parse(e.to_string()))?;
    db_lock
        .clone()
        .ok_or_else(|| TaplootError::Parse("no pcap loaded".into()))
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn get_hosts(state: State<'_, AppState>) -> Result<Vec<Host>, TaplootError> {
    let db = get_db(&state)?;
    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;
    queries::get_all_hosts(&conn)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn get_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, TaplootError> {
    let db = get_db(&state)?;
    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;
    queries::get_all_connections(&conn)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn get_time_range(state: State<'_, AppState>) -> Result<(f64, f64), TaplootError> {
    let db = get_db(&state)?;
    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;
    queries::get_time_range(&conn)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn save_node_position(
    host_id: i64,
    x: f64,
    y: f64,
    state: State<'_, AppState>,
) -> Result<(), TaplootError> {
    let db = get_db(&state)?;
    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;
    queries::save_node_position(&conn, host_id, x, y)
}
