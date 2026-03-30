use rusqlite::{params, Connection};

use crate::commands::query::{Connection as NetConnection, Host};
use crate::TaplootError;

pub fn insert_host(
    conn: &Connection,
    mac: &str,
    ip: &str,
    timestamp: f64,
) -> Result<i64, TaplootError> {
    conn.execute(
        "INSERT INTO hosts (mac_address, ip_address, first_seen, last_seen)
         VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(ip_address) DO UPDATE SET
            last_seen = MAX(last_seen, ?3),
            mac_address = CASE WHEN mac_address = '' THEN ?1 ELSE mac_address END",
        params![mac, ip, timestamp],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_host_timestamp(
    conn: &Connection,
    id: i64,
    timestamp: f64,
) -> Result<(), TaplootError> {
    conn.execute(
        "UPDATE hosts SET
            first_seen = MIN(first_seen, ?1),
            last_seen = MAX(last_seen, ?1)
         WHERE id = ?2",
        params![timestamp, id],
    )?;
    Ok(())
}

pub fn insert_connection(
    conn: &Connection,
    src_host_id: i64,
    dst_host_id: i64,
    src_port: u16,
    dst_port: u16,
    protocol: &str,
    app_protocol: Option<&str>,
    timestamp: f64,
    byte_count: i64,
) -> Result<i64, TaplootError> {
    conn.execute(
        "INSERT INTO connections
            (src_host_id, dst_host_id, src_port, dst_port, protocol, app_protocol,
             packet_count, byte_count, first_seen, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?8)",
        params![
            src_host_id,
            dst_host_id,
            src_port,
            dst_port,
            protocol,
            app_protocol,
            byte_count,
            timestamp,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_connection(
    conn: &Connection,
    id: i64,
    timestamp: f64,
    bytes: i64,
) -> Result<(), TaplootError> {
    conn.execute(
        "UPDATE connections SET
            packet_count = packet_count + 1,
            byte_count = byte_count + ?1,
            first_seen = MIN(first_seen, ?2),
            last_seen = MAX(last_seen, ?2)
         WHERE id = ?3",
        params![bytes, timestamp, id],
    )?;
    Ok(())
}

pub fn insert_packet(
    conn: &Connection,
    connection_id: i64,
    timestamp: f64,
    src_ip: &str,
    dst_ip: &str,
    src_port: u16,
    dst_port: u16,
    protocol: &str,
    length: i64,
) -> Result<(), TaplootError> {
    conn.execute(
        "INSERT INTO packets
            (connection_id, timestamp, src_ip, dst_ip, src_port, dst_port, protocol, length)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            connection_id,
            timestamp,
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
            length
        ],
    )?;
    Ok(())
}

pub fn get_all_hosts(conn: &Connection) -> Result<Vec<Host>, TaplootError> {
    let mut stmt = conn.prepare(
        "SELECT id, mac_address, ip_address, device_type, first_seen, last_seen FROM hosts",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Host {
            id: row.get(0)?,
            mac_address: row.get(1)?,
            ip_address: row.get(2)?,
            device_type: row.get(3)?,
            first_seen: row.get(4)?,
            last_seen: row.get(5)?,
        })
    })?;
    let mut hosts = Vec::new();
    for row in rows {
        hosts.push(row?);
    }
    Ok(hosts)
}

pub fn get_all_connections(conn: &Connection) -> Result<Vec<NetConnection>, TaplootError> {
    let mut stmt = conn.prepare(
        "SELECT id, src_host_id, dst_host_id, src_port, dst_port, protocol,
                app_protocol, packet_count, byte_count, first_seen, last_seen
         FROM connections",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(NetConnection {
            id: row.get(0)?,
            src_host_id: row.get(1)?,
            dst_host_id: row.get(2)?,
            src_port: row.get(3)?,
            dst_port: row.get(4)?,
            protocol: row.get(5)?,
            app_protocol: row.get(6)?,
            packet_count: row.get(7)?,
            byte_count: row.get(8)?,
            first_seen: row.get(9)?,
            last_seen: row.get(10)?,
        })
    })?;
    let mut connections = Vec::new();
    for row in rows {
        connections.push(row?);
    }
    Ok(connections)
}

pub fn get_time_range(conn: &Connection) -> Result<(f64, f64), TaplootError> {
    let mut stmt = conn
        .prepare("SELECT COALESCE(MIN(timestamp), 0), COALESCE(MAX(timestamp), 0) FROM packets")?;
    let range = stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(range)
}

pub fn save_node_position(
    conn: &Connection,
    host_id: i64,
    x: f64,
    y: f64,
) -> Result<(), TaplootError> {
    conn.execute(
        "INSERT INTO node_positions (host_id, x, y) VALUES (?1, ?2, ?3)
         ON CONFLICT(host_id) DO UPDATE SET x = ?2, y = ?3",
        params![host_id, x, y],
    )?;
    Ok(())
}
