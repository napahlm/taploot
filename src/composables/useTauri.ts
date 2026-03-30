import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Host, Connection, ImportResult, HostDetail, Packet } from '@/types/network'
import { useAppStore } from '@/stores/app'
import { useTopologyStore } from '@/stores/topology'
import { useTimelineStore } from '@/stores/timeline'

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

  async function loadFile(path: string) {
    const appStore = useAppStore()
    const topologyStore = useTopologyStore()
    const timelineStore = useTimelineStore()

    appStore.setLoading(true)
    const unlisten = await listen<{ bytes_done: number; bytes_total: number }>(
      'import-progress',
      (event) => {
        if (event.payload.bytes_total > 0) {
          appStore.importProgress = event.payload.bytes_done / event.payload.bytes_total
        }
      },
    )
    try {
      await importPcap(path)
      const [hosts, connections, timeRange] = await Promise.all([
        getHosts(),
        getConnections(),
        getTimeRange(),
      ])
      timelineStore.setFullRange(timeRange[0], timeRange[1])
      topologyStore.buildGraph(hosts, connections)
      appStore.setLoadedFile(path)
    } catch (e) {
      appStore.setError(e instanceof Error ? e.message : String(e))
    } finally {
      unlisten()
      appStore.setLoading(false)
    }
  }

  return {
    importPcap,
    getHosts,
    getConnections,
    getTimeRange,
    saveNodePosition,
    getHostDetail,
    getConnectionPackets,
    loadFile,
  }
}
