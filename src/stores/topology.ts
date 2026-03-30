import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import {
  forceSimulation,
  forceLink,
  forceManyBody,
  forceCenter,
  forceCollide,
  type Simulation,
  type SimulationNodeDatum,
} from 'd3-force'
import type { Host, Connection } from '@/types/network'
import type { CanvasNode, CanvasEdge, LayoutConfig } from '@/types/canvas'
import { useTimelineStore } from './timeline'

type SimNode = CanvasNode & SimulationNodeDatum

const EDGE_COLORS: Record<string, string> = {
  TCP: '#6b7280',
  UDP: '#3b82f6',
  Modbus: '#39ff14',
}

const DEFAULT_LAYOUT: LayoutConfig = {
  linkDistance: 120,
  chargeStrength: -300,
  centerX: 640,
  centerY: 400,
}

function edgeColor(conn: Connection): string {
  if (conn.app_protocol) return EDGE_COLORS[conn.app_protocol] ?? '#6b7280'
  return EDGE_COLORS[conn.protocol] ?? '#6b7280'
}

function edgeWidth(conn: Connection): number {
  const base = Math.log2(conn.packet_count + 1)
  return Math.max(1, Math.min(base, 6))
}

export const useTopologyStore = defineStore('topology', () => {
  const nodes = ref<CanvasNode[]>([])
  const edges = ref<CanvasEdge[]>([])
  const selectedNodeId = ref<number | null>(null)
  const selectedEdgeId = ref<number | null>(null)
  const searchQuery = ref('')
  const layout = ref<LayoutConfig>({ ...DEFAULT_LAYOUT })
  let simulation: Simulation<SimNode, undefined> | null = null
  let onTickCallback: (() => void) | null = null

  const timelineStore = useTimelineStore()

  const filteredEdges = computed(() => {
    if (!timelineStore.filtering) return edges.value
    const { start, end } = timelineStore.filterRange
    return edges.value.filter((e) => e.connection.last_seen >= start && e.connection.first_seen <= end)
  })

  const filteredNodes = computed(() => {
    if (!timelineStore.filtering) return nodes.value
    const activeHostIds = new Set<number>()
    for (const edge of filteredEdges.value) {
      activeHostIds.add(edge.source.host.id)
      activeHostIds.add(edge.target.host.id)
    }
    return nodes.value.filter((n) => activeHostIds.has(n.host.id))
  })

  const matchedNodeIds = computed<Set<number>>(() => {
    const q = searchQuery.value.trim().toLowerCase()
    if (!q) return new Set()
    const matched = new Set<number>()
    for (const node of nodes.value) {
      const h = node.host
      if (
        h.ip_address.toLowerCase().includes(q) ||
        h.mac_address.toLowerCase().includes(q) ||
        h.device_type.toLowerCase().includes(q)
      ) {
        matched.add(h.id)
      }
    }
    // Also match edges by protocol
    for (const edge of edges.value) {
      const c = edge.connection
      const proto = (c.app_protocol ?? c.protocol).toLowerCase()
      if (proto.includes(q)) {
        matched.add(c.src_host_id)
        matched.add(c.dst_host_id)
      }
    }
    return matched
  })

  function buildGraph(hosts: Host[], connections: Connection[]) {
    const nodeMap = new Map<number, CanvasNode>()

    nodes.value = hosts.map((host) => {
      const node: CanvasNode = {
        host,
        x: layout.value.centerX + (Math.random() - 0.5) * 200,
        y: layout.value.centerY + (Math.random() - 0.5) * 200,
        vx: 0,
        vy: 0,
        radius: 18,
        color: '#39ff14',
        label: host.ip_address,
        pinned: false,
      }
      nodeMap.set(host.id, node)
      return node
    })

    edges.value = connections
      .filter((c) => nodeMap.has(c.src_host_id) && nodeMap.has(c.dst_host_id))
      .map((conn) => ({
        connection: conn,
        source: nodeMap.get(conn.src_host_id)!,
        target: nodeMap.get(conn.dst_host_id)!,
        color: edgeColor(conn),
        width: edgeWidth(conn),
      }))

    runSimulation()
  }

  function runSimulation() {
    if (simulation) simulation.stop()

    const simNodes = nodes.value as SimNode[]
    const simLinks = edges.value.map((e) => ({
      source: simNodes.indexOf(e.source as SimNode),
      target: simNodes.indexOf(e.target as SimNode),
    }))

    simulation = forceSimulation(simNodes)
      .force(
        'link',
        forceLink(simLinks).distance(layout.value.linkDistance),
      )
      .force('charge', forceManyBody().strength(layout.value.chargeStrength))
      .force('center', forceCenter(layout.value.centerX, layout.value.centerY))
      .force('collide', forceCollide(24))
      .on('tick', () => {
        // Trigger reactivity by shallow-replacing the array
        nodes.value = [...nodes.value]
        onTickCallback?.()
      })
  }

  function setOnTick(cb: () => void) {
    onTickCallback = cb
  }

  function pinNode(hostId: number, x: number, y: number) {
    const node = nodes.value.find((n) => n.host.id === hostId)
    if (!node) return
    node.pinned = true
    node.x = x
    node.y = y
    const simNode = node as SimNode
    simNode.fx = x
    simNode.fy = y
  }

  function unpinNode(hostId: number) {
    const node = nodes.value.find((n) => n.host.id === hostId)
    if (!node) return
    node.pinned = false
    const simNode = node as SimNode
    simNode.fx = null
    simNode.fy = null
  }

  function selectNode(hostId: number | null) {
    selectedNodeId.value = hostId
    if (hostId !== null) selectedEdgeId.value = null
  }

  function selectEdge(edgeId: number | null) {
    selectedEdgeId.value = edgeId
    if (edgeId !== null) selectedNodeId.value = null
  }

  function clearSelection() {
    selectedNodeId.value = null
    selectedEdgeId.value = null
  }

  function reset() {
    if (simulation) simulation.stop()
    simulation = null
    nodes.value = []
    edges.value = []
    selectedNodeId.value = null
    selectedEdgeId.value = null
    onTickCallback = null
  }

  function updateCenter(cx: number, cy: number) {
    layout.value.centerX = cx
    layout.value.centerY = cy
    if (simulation) {
      simulation.force('center', forceCenter(cx, cy))
      simulation.alpha(0.3).restart()
    }
  }

  return {
    nodes,
    edges,
    selectedNodeId,
    selectedEdgeId,
    searchQuery,
    layout,
    filteredNodes,
    filteredEdges,
    matchedNodeIds,
    buildGraph,
    setOnTick,
    pinNode,
    unpinNode,
    selectNode,
    selectEdge,
    clearSelection,
    reset,
    updateCenter,
  }
})
