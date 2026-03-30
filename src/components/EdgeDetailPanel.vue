<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useTopologyStore } from '@/stores/topology'
import { useTauri } from '@/composables/useTauri'
import type { Packet } from '@/types/network'

const topology = useTopologyStore()
const { getConnectionPackets } = useTauri()

const packets = ref<Packet[]>([])
const loading = ref(false)

const connection = computed(() => {
  if (topology.selectedEdgeId === null) return null
  return topology.edges.find((e) => e.connection.id === topology.selectedEdgeId)?.connection ?? null
})

const srcHost = computed(() => {
  if (!connection.value) return null
  return topology.nodes.find((n) => n.host.id === connection.value!.src_host_id)?.host ?? null
})

const dstHost = computed(() => {
  if (!connection.value) return null
  return topology.nodes.find((n) => n.host.id === connection.value!.dst_host_id)?.host ?? null
})

watch(
  () => topology.selectedEdgeId,
  async (edgeId) => {
    if (edgeId === null) {
      packets.value = []
      return
    }
    loading.value = true
    try {
      packets.value = await getConnectionPackets(edgeId, 100)
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

function formatTime(ts: number): string {
  if (ts <= 0) return '—'
  return new Date(ts * 1000).toLocaleString()
}

function formatPacketTime(ts: number): string {
  if (ts <= 0) return '—'
  const d = new Date(ts * 1000)
  return d.toLocaleTimeString(undefined, { hour12: false, fractionalSecondDigits: 3 })
}

function close() {
  topology.selectEdge(null)
}

function openHost(hostId: number) {
  topology.selectNode(hostId)
}
</script>

<template>
  <div
    class="flex h-full w-80 flex-col border-l border-border bg-bg-secondary"
    @keydown.escape="close"
  >
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-border px-4 py-3">
      <h2 class="text-sm font-semibold text-text-primary">Connection Detail</h2>
      <button
        class="text-text-secondary hover:text-text-primary"
        @click="close"
      >
        ✕
      </button>
    </div>

    <div v-if="loading" class="flex flex-1 items-center justify-center text-text-muted">
      Loading…
    </div>

    <div v-else-if="connection" class="flex-1 overflow-y-auto">
      <!-- Endpoints -->
      <div class="border-b border-border px-4 py-3">
        <div class="mb-2 text-xs font-medium uppercase tracking-wider text-text-muted">Endpoints</div>
        <div class="space-y-2 text-sm">
          <button
            v-if="srcHost"
            class="flex w-full items-center gap-2 rounded px-2 py-1 text-left hover:bg-bg-elevated"
            @click="openHost(srcHost.id)"
          >
            <span class="text-xs text-text-muted">SRC</span>
            <span class="flex-1 font-mono text-text-primary">{{ srcHost.ip_address }}</span>
            <span class="text-xs text-text-secondary">:{{ connection.src_port }}</span>
          </button>
          <button
            v-if="dstHost"
            class="flex w-full items-center gap-2 rounded px-2 py-1 text-left hover:bg-bg-elevated"
            @click="openHost(dstHost.id)"
          >
            <span class="text-xs text-text-muted">DST</span>
            <span class="flex-1 font-mono text-text-primary">{{ dstHost.ip_address }}</span>
            <span class="text-xs text-text-secondary">:{{ connection.dst_port }}</span>
          </button>
        </div>
      </div>

      <!-- Connection info -->
      <div class="border-b border-border px-4 py-3">
        <div class="mb-2 text-xs font-medium uppercase tracking-wider text-text-muted">Details</div>
        <div class="space-y-1.5 text-sm">
          <div class="flex justify-between">
            <span class="text-text-secondary">Protocol</span>
            <span class="text-text-primary">{{ connection.protocol }}</span>
          </div>
          <div v-if="connection.app_protocol" class="flex justify-between">
            <span class="text-text-secondary">App Protocol</span>
            <span class="font-medium text-accent">{{ connection.app_protocol }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-secondary">Packets</span>
            <span class="text-text-primary">{{ connection.packet_count.toLocaleString() }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-secondary">Bytes</span>
            <span class="text-text-primary">{{ formatBytes(connection.byte_count) }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-secondary">First seen</span>
            <span class="text-text-primary">{{ formatTime(connection.first_seen) }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-secondary">Last seen</span>
            <span class="text-text-primary">{{ formatTime(connection.last_seen) }}</span>
          </div>
        </div>
      </div>

      <!-- Packets table -->
      <div class="px-4 py-3">
        <div class="mb-2 text-xs font-medium uppercase tracking-wider text-text-muted">
          Packets ({{ packets.length }}{{ packets.length >= 100 ? '+' : '' }})
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-xs">
            <thead>
              <tr class="border-b border-border text-left text-text-muted">
                <th class="pb-1 pr-2">Time</th>
                <th class="pb-1 pr-2">Src</th>
                <th class="pb-1 pr-2">Dst</th>
                <th class="pb-1 text-right">Len</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="pkt in packets"
                :key="pkt.id"
                class="border-b border-border/30"
              >
                <td class="py-0.5 pr-2 font-mono text-text-secondary">{{ formatPacketTime(pkt.timestamp) }}</td>
                <td class="py-0.5 pr-2 font-mono text-text-primary">{{ pkt.src_port }}</td>
                <td class="py-0.5 pr-2 font-mono text-text-primary">{{ pkt.dst_port }}</td>
                <td class="py-0.5 text-right font-mono text-text-secondary">{{ pkt.length }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
