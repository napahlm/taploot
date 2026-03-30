<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useTopologyStore } from '@/stores/topology'

const topology = useTopologyStore()

const range = computed(() => topology.packetCountRange)

const lo = ref(0)
const hi = ref(0)

watch(range, (r) => {
  lo.value = r.min
  hi.value = r.max
}, { immediate: true })

function onMinChange(e: Event) {
  const v = Number((e.target as HTMLInputElement).value)
  lo.value = Math.min(v, hi.value)
  topology.setTrafficFilter(lo.value, hi.value)
}

function onMaxChange(e: Event) {
  const v = Number((e.target as HTMLInputElement).value)
  hi.value = Math.max(v, lo.value)
  topology.setTrafficFilter(lo.value, hi.value)
}

function resetFilter() {
  lo.value = range.value.min
  hi.value = range.value.max
  topology.setTrafficFilter(0, Infinity)
}

const filtering = computed(() =>
  lo.value > range.value.min || hi.value < range.value.max,
)
</script>

<template>
  <div
    v-if="range.max > 0"
    class="flex items-center gap-2 rounded border border-border bg-bg-secondary px-3 py-1 text-xs text-text-secondary"
  >
    <span class="whitespace-nowrap">Packets</span>
    <input
      type="range"
      :min="range.min"
      :max="range.max"
      :value="lo"
      class="h-1 w-20 accent-accent"
      @input="onMinChange"
    />
    <span class="tabular-nums">{{ lo }}</span>
    <span class="text-text-muted">–</span>
    <input
      type="range"
      :min="range.min"
      :max="range.max"
      :value="hi"
      class="h-1 w-20 accent-accent"
      @input="onMaxChange"
    />
    <span class="tabular-nums">{{ hi }}</span>
    <button
      v-if="filtering"
      class="ml-1 text-accent-dim hover:text-accent"
      @click="resetFilter"
    >
      ✕
    </button>
  </div>
</template>
