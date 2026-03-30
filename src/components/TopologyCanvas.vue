<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import Konva from 'konva'
import { useCanvas } from '@/composables/useCanvas'
import { useTopologyStore } from '@/stores/topology'
import { useTauri } from '@/composables/useTauri'
import { createNodeGroup, updateNodeGroup } from '@/canvas/CanvasNode'
import { createEdgeLine, updateEdgeLine } from '@/canvas/CanvasEdge'

const containerRef = ref<HTMLDivElement | null>(null)
const { mainLayer, getCenter } = useCanvas(containerRef)

const topology = useTopologyStore()
const { saveNodePosition } = useTauri()

const nodeGroups = new Map<number, Konva.Group>()
const edgeLines = new Map<number, Konva.Line>()

function renderGraph() {
  const layer = mainLayer.value
  if (!layer) return

  // Clear old shapes
  for (const g of nodeGroups.values()) g.destroy()
  for (const l of edgeLines.values()) l.destroy()
  nodeGroups.clear()
  edgeLines.clear()

  // Edges first (below nodes)
  for (const edge of topology.filteredEdges) {
    const line = createEdgeLine(edge, {
      onClick(connectionId) {
        topology.selectEdge(connectionId === topology.selectedEdgeId ? null : connectionId)
      },
    })
    layer.add(line)
    edgeLines.set(edge.connection.id, line)
  }

  // Nodes on top
  for (const node of topology.filteredNodes) {
    const group = createNodeGroup(node, {
      onDragMove(hostId, x, y) {
        topology.pinNode(hostId, x, y)
      },
      onDragEnd(hostId, x, y) {
        if (!node.cluster) saveNodePosition(hostId, x, y)
      },
      onClick(hostId) {
        if (node.cluster) {
          topology.toggleCluster(node.cluster.subnet)
        } else {
          topology.selectNode(hostId === topology.selectedNodeId ? null : hostId)
        }
      },
    })
    layer.add(group)
    nodeGroups.set(node.host.id, group)
  }

  layer.batchDraw()
}

function updatePositions() {
  const searching = topology.searchQuery.trim().length > 0
  const matched = topology.matchedNodeIds

  for (const node of topology.filteredNodes) {
    const group = nodeGroups.get(node.host.id)
    if (group) {
      let searchState: 'match' | 'dim' | 'none' = 'none'
      if (searching) {
        searchState = matched.has(node.host.id) ? 'match' : 'dim'
      }
      updateNodeGroup(group, node, node.host.id === topology.selectedNodeId, searchState)
    }
  }

  for (const edge of topology.filteredEdges) {
    const line = edgeLines.get(edge.connection.id)
    if (line) {
      updateEdgeLine(line, edge, edge.connection.id === topology.selectedEdgeId)
    }
  }

  mainLayer.value?.batchDraw()
}

// Update d3 center when canvas center changes
onMounted(() => {
  const center = getCenter()
  topology.updateCenter(center.x, center.y)
})

// Re-render when the graph data changes
watch(
  () => [topology.filteredNodes.length, topology.filteredEdges.length],
  () => renderGraph(),
)

// Update positions on simulation tick
topology.setOnTick(() => updatePositions())

// Update selection styling
watch(
  () => topology.selectedNodeId,
  () => updatePositions(),
)

watch(
  () => topology.selectedEdgeId,
  () => updatePositions(),
)

watch(
  () => topology.searchQuery,
  () => updatePositions(),
)
</script>

<template>
  <div ref="containerRef" class="h-full w-full bg-bg-primary" />
</template>
