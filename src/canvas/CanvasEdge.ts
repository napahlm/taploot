import Konva from 'konva'
import type { CanvasEdge } from '@/types/canvas'

export function createEdgeLine(edge: CanvasEdge): Konva.Line {
  return new Konva.Line({
    points: [edge.source.x, edge.source.y, edge.target.x, edge.target.y],
    stroke: edge.color,
    strokeWidth: edge.width,
    opacity: 0.6,
    hitStrokeWidth: 10,
    id: `edge-${edge.connection.id}`,
  })
}

export function updateEdgeLine(line: Konva.Line, edge: CanvasEdge) {
  line.points([edge.source.x, edge.source.y, edge.target.x, edge.target.y])
}
