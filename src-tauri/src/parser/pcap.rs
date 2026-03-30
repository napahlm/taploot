use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::*;
use rusqlite::params;

use crate::commands::import::ImportResult;
use crate::db::{queries, schema};
use crate::oui;
use crate::protocols::modbus;
use crate::TaplootError;

/// Buffered packet row for batch insert
struct PacketRow {
    connection_id: i64,
    timestamp: f64,
    src_ip: String,
    dst_ip: String,
    src_port: u16,
    dst_port: u16,
    protocol: String,
    length: i64,
}

const PACKET_BATCH_SIZE: usize = 5000;

pub fn parse_pcap(
    path: &Path,
    db: &Arc<Mutex<rusqlite::Connection>>,
) -> Result<ImportResult, TaplootError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;

    // Clear stale data and drop indexes for fast bulk insert
    schema::clear_data(&conn)?;
    schema::drop_packet_indexes(&conn)?;

    // Single transaction for the entire import
    conn.execute_batch("BEGIN EXCLUSIVE")?;

    let mut host_map: HashMap<String, i64> = HashMap::new();
    let mut conn_map: HashMap<String, i64> = HashMap::new();
    let mut packet_buf: Vec<PacketRow> = Vec::with_capacity(PACKET_BATCH_SIZE);
    let mut packet_count: usize = 0;
    let mut min_ts: f64 = f64::MAX;
    let mut max_ts: f64 = f64::MIN;

    if buf.len() < 4 {
        conn.execute_batch("ROLLBACK")?;
        return Err(TaplootError::Parse("file too small".into()));
    }

    let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let is_pcapng = magic == 0x0A0D_0D0A;

    let result = if is_pcapng {
        parse_pcapng_data(
            &buf, &conn, &mut host_map, &mut conn_map,
            &mut packet_buf, &mut packet_count, &mut min_ts, &mut max_ts,
        )
    } else {
        parse_legacy_data(
            &buf, &conn, &mut host_map, &mut conn_map,
            &mut packet_buf, &mut packet_count, &mut min_ts, &mut max_ts,
        )
    };

    if let Err(e) = result {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(e);
    }

    // Flush remaining buffered packets
    if !packet_buf.is_empty() {
        flush_packets(&conn, &packet_buf)?;
    }

    // Recreate indexes after all data is inserted
    schema::create_packet_indexes(&conn)?;

    conn.execute_batch("COMMIT")?;

    if packet_count == 0 {
        min_ts = 0.0;
        max_ts = 0.0;
    }

    Ok(ImportResult {
        host_count: host_map.len(),
        connection_count: conn_map.len(),
        packet_count,
        time_range: (min_ts, max_ts),
    })
}

/// Batch-insert buffered packets using a multi-value INSERT
fn flush_packets(conn: &rusqlite::Connection, rows: &[PacketRow]) -> Result<(), TaplootError> {
    if rows.is_empty() {
        return Ok(());
    }
    // Build multi-value INSERT: INSERT INTO packets (...) VALUES (?,?,?,...), (?,?,?,...),...
    let mut sql = String::from(
        "INSERT INTO packets (connection_id, timestamp, src_ip, dst_ip, src_port, dst_port, protocol, length) VALUES ",
    );
    for (i, _) in rows.iter().enumerate() {
        if i > 0 {
            sql.push(',');
        }
        sql.push_str("(?,?,?,?,?,?,?,?)");
    }

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut idx = 1;
    for row in rows {
        stmt.raw_bind_parameter(idx, row.connection_id)?;
        stmt.raw_bind_parameter(idx + 1, row.timestamp)?;
        stmt.raw_bind_parameter(idx + 2, &row.src_ip)?;
        stmt.raw_bind_parameter(idx + 3, &row.dst_ip)?;
        stmt.raw_bind_parameter(idx + 4, i64::from(row.src_port))?;
        stmt.raw_bind_parameter(idx + 5, i64::from(row.dst_port))?;
        stmt.raw_bind_parameter(idx + 6, &row.protocol)?;
        stmt.raw_bind_parameter(idx + 7, row.length)?;
        idx += 8;
    }
    stmt.raw_execute()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_pcapng_data(
    data: &[u8],
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    conn_map: &mut HashMap<String, i64>,
    packet_buf: &mut Vec<PacketRow>,
    packet_count: &mut usize,
    min_ts: &mut f64,
    max_ts: &mut f64,
) -> Result<(), TaplootError> {
    let mut reader = PcapNGReader::new(65536, std::io::Cursor::new(data))
        .map_err(|e| TaplootError::Parse(format!("pcapng reader: {e}")))?;

    let mut if_info: Vec<(u64, u64)> = Vec::new();

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                match block {
                    PcapBlockOwned::NG(Block::InterfaceDescription(idb)) => {
                        let resolution = idb.ts_resolution().unwrap_or(1_000_000);
                        let ts_offset = idb.if_tsoffset as u64;
                        if_info.push((ts_offset, resolution));
                    }
                    PcapBlockOwned::NG(Block::EnhancedPacket(epb)) => {
                        let (ts_offset, resolution) = if_info
                            .get(epb.if_id as usize)
                            .copied()
                            .unwrap_or((0, 1_000_000));
                        let ts = epb.decode_ts_f64(ts_offset, resolution);
                        process_packet(
                            epb.data, ts, conn, host_map, conn_map,
                            packet_buf, packet_count, min_ts, max_ts,
                        )?;
                    }
                    PcapBlockOwned::NG(Block::SimplePacket(spb)) => {
                        process_packet(
                            spb.data, 0.0, conn, host_map, conn_map,
                            packet_buf, packet_count, min_ts, max_ts,
                        )?;
                    }
                    _ => {}
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader
                    .refill()
                    .map_err(|e| TaplootError::Parse(format!("refill: {e}")))?;
            }
            Err(e) => return Err(TaplootError::Parse(format!("pcapng: {e}"))),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_legacy_data(
    data: &[u8],
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    conn_map: &mut HashMap<String, i64>,
    packet_buf: &mut Vec<PacketRow>,
    packet_count: &mut usize,
    min_ts: &mut f64,
    max_ts: &mut f64,
) -> Result<(), TaplootError> {
    let mut reader = LegacyPcapReader::new(65536, std::io::Cursor::new(data))
        .map_err(|e| TaplootError::Parse(format!("pcap reader: {e}")))?;

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                if let PcapBlockOwned::Legacy(packet) = block {
                    let ts = f64::from(packet.ts_sec) + f64::from(packet.ts_usec) / 1_000_000.0;
                    process_packet(
                        packet.data, ts, conn, host_map, conn_map,
                        packet_buf, packet_count, min_ts, max_ts,
                    )?;
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader
                    .refill()
                    .map_err(|e| TaplootError::Parse(format!("refill: {e}")))?;
            }
            Err(e) => return Err(TaplootError::Parse(format!("pcap: {e}"))),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_packet(
    data: &[u8],
    timestamp: f64,
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    conn_map: &mut HashMap<String, i64>,
    packet_buf: &mut Vec<PacketRow>,
    packet_count: &mut usize,
    min_ts: &mut f64,
    max_ts: &mut f64,
) -> Result<(), TaplootError> {
    let Ok(parsed) = etherparse::SlicedPacket::from_ethernet(data) else {
        return Ok(());
    };

    let (src_mac, dst_mac) = if data.len() >= 14 {
        (format_mac(&data[6..12]), format_mac(&data[0..6]))
    } else {
        return Ok(());
    };

    let (src_ip, dst_ip) = match &parsed.net {
        Some(etherparse::NetSlice::Ipv4(ipv4)) => {
            let h = ipv4.header();
            (
                format!("{}", h.source_addr()),
                format!("{}", h.destination_addr()),
            )
        }
        _ => return Ok(()),
    };

    let (src_port, dst_port, protocol, payload): (u16, u16, &str, &[u8]) = match &parsed.transport {
        Some(etherparse::TransportSlice::Tcp(tcp)) => (
            tcp.source_port(),
            tcp.destination_port(),
            "TCP",
            tcp.payload(),
        ),
        Some(etherparse::TransportSlice::Udp(udp)) => (
            udp.source_port(),
            udp.destination_port(),
            "UDP",
            udp.payload(),
        ),
        Some(etherparse::TransportSlice::Icmpv4(_)) => (0, 0, "ICMP", &[] as &[u8]),
        _ => return Ok(()),
    };

    let app_protocol = if protocol == "TCP" && modbus::is_modbus_tcp(src_port, dst_port, payload) {
        Some("modbus".to_string())
    } else {
        None
    };

    if timestamp > 0.0 {
        if timestamp < *min_ts {
            *min_ts = timestamp;
        }
        if timestamp > *max_ts {
            *max_ts = timestamp;
        }
    }

    // Upsert hosts — in-memory cache avoids repeated DB calls
    let src_host_id = upsert_host(conn, host_map, &src_ip, &src_mac, timestamp)?;
    let dst_host_id = upsert_host(conn, host_map, &dst_ip, &dst_mac, timestamp)?;

    // Upsert connection — in-memory cache for the common case
    let flow_key = format!("{src_ip}:{src_port}-{dst_ip}:{dst_port}-{protocol}");
    let protocol = protocol.to_string();
    let conn_id = if let Some(&id) = conn_map.get(&flow_key) {
        conn.prepare_cached(
            "UPDATE connections SET
                packet_count = packet_count + 1,
                byte_count = byte_count + ?1,
                first_seen = MIN(first_seen, ?2),
                last_seen = MAX(last_seen, ?2)
             WHERE id = ?3",
        )?
        .execute(params![data.len() as i64, timestamp, id])?;
        id
    } else {
        conn.prepare_cached(
            "INSERT INTO connections
                (src_host_id, dst_host_id, src_port, dst_port, protocol, app_protocol,
                 packet_count, byte_count, first_seen, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?8)",
        )?
        .execute(params![
            src_host_id, dst_host_id, src_port, dst_port,
            &protocol, app_protocol.as_deref(),
            data.len() as i64, timestamp,
        ])?;
        let id = conn.last_insert_rowid();
        conn_map.insert(flow_key, id);
        id
    };

    // Buffer packet for batch insert
    packet_buf.push(PacketRow {
        connection_id: conn_id,
        timestamp,
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol,
        length: data.len() as i64,
    });

    if packet_buf.len() >= PACKET_BATCH_SIZE {
        flush_packets(conn, packet_buf)?;
        packet_buf.clear();
    }

    *packet_count += 1;
    Ok(())
}

fn upsert_host(
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    ip: &str,
    mac: &str,
    timestamp: f64,
) -> Result<i64, TaplootError> {
    if let Some(&id) = host_map.get(ip) {
        conn.prepare_cached(
            "UPDATE hosts SET
                first_seen = MIN(first_seen, ?1),
                last_seen = MAX(last_seen, ?1)
             WHERE id = ?2",
        )?
        .execute(params![timestamp, id])?;
        return Ok(id);
    }
    let id = queries::upsert_host_returning_id(conn, mac, ip, timestamp)?;
    if let Some(vendor) = oui::lookup_vendor(mac) {
        conn.prepare_cached(
            "UPDATE hosts SET device_type = ?1 WHERE id = ?2 AND device_type = 'unknown'",
        )?
        .execute(params![vendor, id])?;
    }
    host_map.insert(ip.to_string(), id);
    Ok(id)
}

fn format_mac(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}
