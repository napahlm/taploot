import Konva from 'konva'
import type { CanvasNode } from '@/types/canvas'

const NODE_RADIUS = 18
const LABEL_OFFSET = 24

export function createNodeGroup(
  node: CanvasNode,
  callbacks: {
    onDragMove: (hostId: number, x: number, y: number) => void
    onDragEnd: (hostId: number, x: number, y: number) => void
    onClick: (hostId: number) => void
  },
): Konva.Group {
  const group = new Konva.Group({
    x: node.x,
    y: node.y,
    draggable: true,
    id: `node-${node.host.id}`,
  })

  const circle = new Konva.Circle({
    radius: NODE_RADIUS,
    fill: '#252545',
    stroke: node.color,
    strokeWidth: 2,
    shadowColor: node.color,
    shadowBlur: 8,
    shadowOpacity: 0.4,
  })

  const label = new Konva.Text({
    text: node.label,
    fontSize: 11,
    fontFamily: 'Segoe UI, system-ui, sans-serif',
    fill: '#e0e0e0',
    align: 'center',
    y: LABEL_OFFSET,
  })
  // Center the label horizontally
  label.x(-label.width() / 2)

  group.add(circle)
  group.add(label)

  group.on('dragmove', () => {
    callbacks.onDragMove(node.host.id, group.x(), group.y())
  })

  group.on('dragend', () => {
    callbacks.onDragEnd(node.host.id, group.x(), group.y())
  })

  group.on('click tap', () => {
    callbacks.onClick(node.host.id)
  })

  return group
}

export function updateNodeGroup(group: Konva.Group, node: CanvasNode, selected: boolean) {
  group.x(node.x)
  group.y(node.y)

  const circle = group.findOne('Circle') as Konva.Circle | undefined
  if (circle) {
    circle.stroke(selected ? '#ffffff' : node.color)
    circle.strokeWidth(selected ? 3 : 2)
    circle.shadowBlur(selected ? 14 : 8)
  }
}
