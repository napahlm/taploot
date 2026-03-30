import { invoke } from '@tauri-apps/api/core'
import type { Host, Connection, ImportResult } from '@/types/network'

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

  return { importPcap, getHosts, getConnections, getTimeRange, saveNodePosition }
}
