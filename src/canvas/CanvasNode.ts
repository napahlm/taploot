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
  const isCluster = !!node.cluster
  const radius = isCluster ? node.radius : NODE_RADIUS

  const group = new Konva.Group({
    x: node.x,
    y: node.y,
    draggable: true,
    id: `node-${node.host.id}`,
  })

  const circle = new Konva.Circle({
    radius,
    fill: isCluster ? '#1e1b4b' : '#252545',
    stroke: node.color,
    strokeWidth: isCluster ? 3 : 2,
    shadowColor: node.color,
    shadowBlur: isCluster ? 12 : 8,
    shadowOpacity: 0.4,
    dash: isCluster ? [6, 3] : undefined,
  })

  const label = new Konva.Text({
    text: node.label,
    fontSize: 11,
    fontFamily: 'Segoe UI, system-ui, sans-serif',
    fill: '#e0e0e0',
    align: 'center',
    y: radius + 6,
  })
  label.x(-label.width() / 2)

  group.add(circle)
  group.add(label)

  if (isCluster) {
    const countText = new Konva.Text({
      text: String(node.cluster!.hostCount),
      fontSize: 14,
      fontFamily: 'Segoe UI, system-ui, sans-serif',
      fontStyle: 'bold',
      fill: '#c4b5fd',
      align: 'center',
    })
    countText.x(-countText.width() / 2)
    countText.y(-countText.height() / 2)
    group.add(countText)
  }

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

export function updateNodeGroup(group: Konva.Group, node: CanvasNode, selected: boolean, searchState?: 'match' | 'dim' | 'none') {
  group.x(node.x)
  group.y(node.y)

  const circle = group.findOne('Circle') as Konva.Circle | undefined
  if (circle) {
    circle.stroke(selected ? '#ffffff' : node.color)
    circle.strokeWidth(selected ? 3 : 2)

    if (searchState === 'match') {
      circle.shadowBlur(20)
      circle.shadowColor('#39ff14')
      circle.shadowOpacity(0.8)
      group.opacity(1)
    } else if (searchState === 'dim') {
      circle.shadowBlur(0)
      group.opacity(0.2)
    } else {
      circle.shadowBlur(selected ? 14 : 8)
      circle.shadowColor(node.color)
      circle.shadowOpacity(0.4)
      group.opacity(1)
    }
  }
}
