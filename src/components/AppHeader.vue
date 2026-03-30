<script setup lang="ts">
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

function fileName(path: string): string {
  const sep = path.includes('\\') ? '\\' : '/'
  return path.split(sep).pop() ?? path
}
</script>

<template>
  <header class="flex h-10 shrink-0 items-center justify-between border-b border-border bg-bg-secondary px-4">
    <span class="text-sm font-bold tracking-wider text-accent">taploot</span>
    <span v-if="appStore.loadedFile" class="text-xs text-text-secondary">
      {{ fileName(appStore.loadedFile) }}
    </span>
  </header>
</template>
