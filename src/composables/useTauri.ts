import { invoke } from '@tauri-apps/api/core'
import type { Host, Connection, ImportResult, HostDetail, Packet } from '@/types/network'

export function useTauri() {
  async function importPcap(path: string): Promise<ImportResult> {
    return invoke<ImportResult>('import_pcap', { path })
  }

  async function getHosts(): Promise<Host[]> {
    return invoke<Host[]>('get_hosts')
  }

  async function getConnections(): Promise<Connection[]> {
    return invoke<Connection[]>('get_connections')
  }

  async function getTimeRange(): Promise<[number, number]> {
    return invoke<[number, number]>('get_time_range')
  }

  async function saveNodePosition(hostId: number, x: number, y: number): Promise<void> {
    return invoke<void>('save_node_position', { hostId, x, y })
  }

  async function getHostDetail(hostId: number): Promise<HostDetail> {
    return invoke<HostDetail>('get_host_detail', { hostId })
  }

  async function getConnectionPackets(connectionId: number, limit: number): Promise<Packet[]> {
    return invoke<Packet[]>('get_connection_packets', { connectionId, limit })
  }

  return {
    importPcap,
    getHosts,
    getConnections,
    getTimeRange,
    saveNodePosition,
    getHostDetail,
    getConnectionPackets,
  }
}
