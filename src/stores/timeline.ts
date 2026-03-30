import { ref } from 'vue'
import { defineStore } from 'pinia'

export interface TimeWindow {
  start: number
  end: number
}

export const useTimelineStore = defineStore('timeline', () => {
  const fullRange = ref<TimeWindow>({ start: 0, end: 0 })
  const filterRange = ref<TimeWindow>({ start: 0, end: 0 })
  const filtering = ref(false)

  function setFullRange(start: number, end: number) {
    fullRange.value = { start, end }
    filterRange.value = { start, end }
    filtering.value = false
  }

  function setFilterRange(start: number, end: number) {
    filterRange.value = { start, end }
    filtering.value = start !== fullRange.value.start || end !== fullRange.value.end
  }

  function resetFilter() {
    filterRange.value = { ...fullRange.value }
    filtering.value = false
  }

  return { fullRange, filterRange, filtering, setFullRange, setFilterRange, resetFilter }
})
