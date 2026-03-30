import Konva from 'konva'
import type { CanvasEdge } from '@/types/canvas'

export function createEdgeLine(
  edge: CanvasEdge,
  callbacks?: { onClick?: (connectionId: number) => void },
): Konva.Line {
  const line = new Konva.Line({
    points: [edge.source.x, edge.source.y, edge.target.x, edge.target.y],
    stroke: edge.color,
    strokeWidth: edge.width,
    opacity: 0.6,
    hitStrokeWidth: 14,
    id: `edge-${edge.connection.id}`,
  })

  if (callbacks?.onClick) {
    const cb = callbacks.onClick
    line.on('click tap', () => cb(edge.connection.id))
  }

  return line
}

export function updateEdgeLine(line: Konva.Line, edge: CanvasEdge, selected: boolean) {
  line.points([edge.source.x, edge.source.y, edge.target.x, edge.target.y])
  line.stroke(selected ? '#ffffff' : edge.color)
  line.strokeWidth(selected ? edge.width + 2 : edge.width)
  line.opacity(selected ? 1 : 0.6)
}
