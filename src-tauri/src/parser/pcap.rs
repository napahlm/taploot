use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::*;

use crate::commands::import::ImportResult;
use crate::db::queries;
use crate::oui;
use crate::protocols::modbus;
use crate::TaplootError;

pub fn parse_pcap(
    path: &Path,
    db: &Arc<Mutex<rusqlite::Connection>>,
) -> Result<ImportResult, TaplootError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let conn = db.lock().map_err(|e| TaplootError::Parse(e.to_string()))?;

    // track hosts and connections we've seen
    let mut host_map: HashMap<String, i64> = HashMap::new();
    let mut conn_map: HashMap<String, i64> = HashMap::new();
    let mut packet_count: usize = 0;
    let mut min_ts: f64 = f64::MAX;
    let mut max_ts: f64 = f64::MIN;

    // detect format by first 4 bytes
    if buf.len() < 4 {
        return Err(TaplootError::Parse("file too small".into()));
    }

    let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let is_pcapng = magic == 0x0A0D_0D0A;

    if is_pcapng {
        parse_pcapng_data(
            &buf,
            &conn,
            &mut host_map,
            &mut conn_map,
            &mut packet_count,
            &mut min_ts,
            &mut max_ts,
        )?;
    } else {
        parse_legacy_data(
            &buf,
            &conn,
            &mut host_map,
            &mut conn_map,
            &mut packet_count,
            &mut min_ts,
            &mut max_ts,
        )?;
    }

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

fn parse_pcapng_data(
    data: &[u8],
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    conn_map: &mut HashMap<String, i64>,
    packet_count: &mut usize,
    min_ts: &mut f64,
    max_ts: &mut f64,
) -> Result<(), TaplootError> {
    let mut reader = PcapNGReader::new(65536, std::io::Cursor::new(data))
        .map_err(|e| TaplootError::Parse(format!("pcapng reader: {e}")))?;

    // track per-interface timestamp resolution and offset
    let mut if_info: Vec<(u64, u64)> = Vec::new(); // (ts_offset, resolution)

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
                            epb.data,
                            ts,
                            conn,
                            host_map,
                            conn_map,
                            packet_count,
                            min_ts,
                            max_ts,
                        )?;
                    }
                    PcapBlockOwned::NG(Block::SimplePacket(spb)) => {
                        process_packet(
                            spb.data,
                            0.0,
                            conn,
                            host_map,
                            conn_map,
                            packet_count,
                            min_ts,
                            max_ts,
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

fn parse_legacy_data(
    data: &[u8],
    conn: &rusqlite::Connection,
    host_map: &mut HashMap<String, i64>,
    conn_map: &mut HashMap<String, i64>,
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
                        packet.data,
                        ts,
                        conn,
                        host_map,
                        conn_map,
                        packet_count,
                        min_ts,
                        max_ts,
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
    packet_count: &mut usize,
    min_ts: &mut f64,
    max_ts: &mut f64,
) -> Result<(), TaplootError> {
    let Ok(parsed) = etherparse::SlicedPacket::from_ethernet(data) else {
        return Ok(()); // skip unparseable packets
    };

    // extract MAC addresses from ethernet header
    let (src_mac, dst_mac) = if data.len() >= 14 {
        (format_mac(&data[6..12]), format_mac(&data[0..6]))
    } else {
        return Ok(());
    };

    // extract IP addresses
    let (src_ip, dst_ip) = match &parsed.net {
        Some(etherparse::NetSlice::Ipv4(ipv4)) => {
            let h = ipv4.header();
            (
                format!("{}", h.source_addr()),
                format!("{}", h.destination_addr()),
            )
        }
        _ => return Ok(()), // skip non-IPv4 for now
    };

    // extract transport layer info
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

    // check for Modbus TCP
    let app_protocol = if protocol == "TCP" && modbus::is_modbus_tcp(src_port, dst_port, payload) {
        Some("modbus".to_string())
    } else {
        None
    };

    // update timestamps
    if timestamp > 0.0 {
        if timestamp < *min_ts {
            *min_ts = timestamp;
        }
        if timestamp > *max_ts {
            *max_ts = timestamp;
        }
    }

    // upsert source host
    let src_host_id = upsert_host(conn, host_map, &src_ip, &src_mac, timestamp)?;
    let dst_host_id = upsert_host(conn, host_map, &dst_ip, &dst_mac, timestamp)?;

    // upsert connection
    let flow_key = format!("{src_ip}:{src_port}-{dst_ip}:{dst_port}-{protocol}");
    let protocol = protocol.to_string();
    let conn_id = if let Some(&id) = conn_map.get(&flow_key) {
        queries::update_connection(conn, id, timestamp, data.len() as i64)?;
        id
    } else {
        let id = queries::insert_connection(
            conn,
            src_host_id,
            dst_host_id,
            src_port,
            dst_port,
            &protocol,
            app_protocol.as_deref(),
            timestamp,
            data.len() as i64,
        )?;
        conn_map.insert(flow_key, id);
        id
    };

    // insert packet summary
    let len = data.len() as i64;
    queries::insert_packet(
        conn, conn_id, timestamp, &src_ip, &dst_ip, src_port, dst_port, &protocol, len,
    )?;

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
        queries::update_host_timestamp(conn, id, timestamp)?;
        return Ok(id);
    }
    let id = queries::insert_host(conn, mac, ip, timestamp)?;
    if let Some(vendor) = oui::lookup_vendor(mac) {
        queries::update_host_device_type(conn, id, vendor)?;
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
