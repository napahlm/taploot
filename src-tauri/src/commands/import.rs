use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::db::schema;
use crate::parser::pcap;
use crate::{AppState, TaplootError};

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub host_count: usize,
    pub connection_count: usize,
    pub packet_count: usize,
    pub time_range: (f64, f64),
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn import_pcap(path: String, state: State<'_, AppState>) -> Result<ImportResult, TaplootError> {
    let pcap_path = PathBuf::from(&path);

    let conn = schema::init_db()?;
    let conn = Arc::new(Mutex::new(conn));

    let result = pcap::parse_pcap(&pcap_path, &conn)?;

    let mut db_lock = state
        .db
        .lock()
        .map_err(|e| TaplootError::Parse(e.to_string()))?;
    *db_lock = Some(conn);

    Ok(result)
}
