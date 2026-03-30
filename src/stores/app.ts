import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', () => {
  const loading = ref(false)
  const loadedFile = ref<string | null>(null)
  const error = ref<string | null>(null)
  const importProgress = ref(0) // 0.0 – 1.0

  function setLoading(value: boolean) {
    loading.value = value
    if (value) importProgress.value = 0
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
    importProgress.value = 0
  }

  return { loading, loadedFile, error, importProgress, setLoading, setLoadedFile, setError, reset }
})
