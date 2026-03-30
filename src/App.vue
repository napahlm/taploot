<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useTopologyStore } from '@/stores/topology'
import FileDropZone from '@/components/FileDropZone.vue'
import AppHeader from '@/components/AppHeader.vue'
import TopologyCanvas from '@/components/TopologyCanvas.vue'
import TimelineBar from '@/components/TimelineBar.vue'
import NodeDetailPanel from '@/components/NodeDetailPanel.vue'
import EdgeDetailPanel from '@/components/EdgeDetailPanel.vue'
import SearchBar from '@/components/SearchBar.vue'
import EdgeFilterBar from '@/components/EdgeFilterBar.vue'

const appStore = useAppStore()
const topology = useTopologyStore()

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    topology.clearSelection()
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="flex h-screen w-screen flex-col bg-bg-primary">
    <template v-if="appStore.loadedFile">
      <AppHeader />
      <div class="relative flex flex-1 overflow-hidden">
        <TopologyCanvas />
        <NodeDetailPanel v-if="topology.selectedNodeId !== null" />
        <EdgeDetailPanel v-if="topology.selectedEdgeId !== null" />
        <div class="absolute bottom-2 left-2 z-10">
          <EdgeFilterBar />
        </div>
      </div>
      <TimelineBar />
      <SearchBar />
    </template>
    <FileDropZone v-else />
  </div>
</template>
