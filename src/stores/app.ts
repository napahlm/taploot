import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', () => {
  const loading = ref(false)
  const loadedFile = ref<string | null>(null)
  const error = ref<string | null>(null)

  function setLoading(value: boolean) {
    loading.value = value
  }

  function setLoadedFile(path: string) {
    loadedFile.value = path
    error.value = null
  }

  function setError(message: string) {
    error.value = message
    loading.value = false
  }

  function reset() {
    loading.value = false
    loadedFile.value = null
    error.value = null
  }

  return { loading, loadedFile, error, setLoading, setLoadedFile, setError, reset }
})
