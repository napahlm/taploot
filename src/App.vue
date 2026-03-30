<script setup lang="ts">
import { useAppStore } from '@/stores/app'
import FileDropZone from '@/components/FileDropZone.vue'
import AppHeader from '@/components/AppHeader.vue'
import TopologyCanvas from '@/components/TopologyCanvas.vue'

const appStore = useAppStore()
</script>

<template>
  <div class="flex h-screen w-screen flex-col bg-bg-primary">
    <template v-if="appStore.loadedFile">
      <AppHeader />
      <div class="flex flex-1 overflow-hidden">
        <TopologyCanvas />
      </div>
    </template>
    <FileDropZone v-else />
  </div>
</template>
