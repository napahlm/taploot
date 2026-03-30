<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { open } from '@tauri-apps/plugin-dialog'
import { useTauri } from '@/composables/useTauri'
import { useAppStore } from '@/stores/app'
import { useTopologyStore } from '@/stores/topology'
import { useTimelineStore } from '@/stores/timeline'

const appStore = useAppStore()
const topologyStore = useTopologyStore()
const timelineStore = useTimelineStore()
const { importPcap, getHosts, getConnections, getTimeRange } = useTauri()

const hovering = ref(false)

async function loadFile(path: string) {
  appStore.setLoading(true)
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
    appStore.setLoading(false)
  }
}

async function openFilePicker() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'PCAP files', extensions: ['pcap', 'pcapng', 'cap'] }],
  })
  if (selected) await loadFile(selected)
}

let unlisten: (() => void) | null = null

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow()
  unlisten = await appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'over') {
      hovering.value = true
    } else if (event.payload.type === 'drop') {
      hovering.value = false
      const paths = event.payload.paths
      if (paths.length > 0) loadFile(paths[0])
    } else {
      hovering.value = false
    }
  })
})

onUnmounted(() => {
  unlisten?.()
})
</script>

<template>
  <div
    class="flex h-full w-full flex-col items-center justify-center transition-colors"
    :class="hovering ? 'bg-bg-surface/40' : ''"
  >
    <div
      class="flex flex-col items-center gap-6 rounded-xl border-2 border-dashed p-12 transition-colors"
      :class="hovering ? 'border-accent bg-bg-elevated/50' : 'border-border'"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-16 w-16 text-text-muted"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M9 8.25H7.5a2.25 2.25 0 00-2.25 2.25v9a2.25 2.25 0 002.25 2.25h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25H15m0-3l-3-3m0 0l-3 3m3-3v11.25"
        />
      </svg>

      <div class="text-center">
        <h1 class="mb-2 text-2xl font-bold text-accent">taploot</h1>
        <p class="text-text-secondary">drop a .pcap file here</p>
        <p class="mt-1 text-sm text-text-muted">or</p>
      </div>

      <button
        class="rounded-lg border border-accent bg-transparent px-6 py-2 text-accent transition-colors hover:bg-accent hover:text-bg-primary"
        @click="openFilePicker"
      >
        browse files
      </button>

      <p v-if="appStore.loading" class="flex flex-col items-center gap-2 text-sm text-accent-dim">
        <span>importing...</span>
        <span class="h-1 w-48 overflow-hidden rounded-full bg-bg-elevated">
          <span class="block h-full animate-pulse rounded-full bg-accent" style="width: 100%"></span>
        </span>
      </p>
      <p v-if="appStore.error" class="max-w-xs text-center text-sm text-red-400">
        {{ appStore.error }}
      </p>
    </div>
  </div>
</template>
