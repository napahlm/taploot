import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import Konva from 'konva'

export function useCanvas(containerRef: Ref<HTMLDivElement | null>) {
  const stage = ref<Konva.Stage | null>(null)
  const mainLayer = ref<Konva.Layer | null>(null)
  const scale = ref(1)

  const MIN_SCALE = 0.1
  const MAX_SCALE = 5

  function init() {
    const el = containerRef.value
    if (!el) return

    const s = new Konva.Stage({
      container: el,
      width: el.clientWidth,
      height: el.clientHeight,
      draggable: true,
    })

    const layer = new Konva.Layer()
    s.add(layer)

    s.on('wheel', (e) => {
      e.evt.preventDefault()
      const oldScale = s.scaleX()
      const pointer = s.getPointerPosition()
      if (!pointer) return

      const direction = e.evt.deltaY > 0 ? -1 : 1
      const factor = 1.08
      const newScale = Math.max(
        MIN_SCALE,
        Math.min(MAX_SCALE, direction > 0 ? oldScale * factor : oldScale / factor),
      )

      const mousePointTo = {
        x: (pointer.x - s.x()) / oldScale,
        y: (pointer.y - s.y()) / oldScale,
      }

      s.scale({ x: newScale, y: newScale })
      s.position({
        x: pointer.x - mousePointTo.x * newScale,
        y: pointer.y - mousePointTo.y * newScale,
      })

      scale.value = newScale
    })

    stage.value = s
    mainLayer.value = layer
  }

  function resize() {
    const el = containerRef.value
    const s = stage.value
    if (!el || !s) return
    s.width(el.clientWidth)
    s.height(el.clientHeight)
  }

  function getCenter(): { x: number; y: number } {
    const s = stage.value
    if (!s) return { x: 640, y: 400 }
    const sc = s.scaleX()
    return {
      x: (-s.x() + s.width() / 2) / sc,
      y: (-s.y() + s.height() / 2) / sc,
    }
  }

  onMounted(() => {
    init()
    window.addEventListener('resize', resize)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', resize)
    stage.value?.destroy()
    stage.value = null
    mainLayer.value = null
  })

  return { stage, mainLayer, scale, resize, getCenter }
}
