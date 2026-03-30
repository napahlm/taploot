export interface Host {
  id: number
  mac_address: string
  ip_address: string
  device_type: string
  first_seen: number
  last_seen: number
}

export interface Connection {
  id: number
  src_host_id: number
  dst_host_id: number
  src_port: number
  dst_port: number
  protocol: string
  app_protocol: string | null
  packet_count: number
  byte_count: number
  first_seen: number
  last_seen: number
}

export interface ImportResult {
  host_count: number
  connection_count: number
  packet_count: number
  time_range: [number, number]
}

export interface TimeRange {
  start: number
  end: number
}
