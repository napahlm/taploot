import type { Host, Connection } from './network'

export interface CanvasNode {
  host: Host
  x: number
  y: number
  vx: number
  vy: number
  radius: number
  color: string
  label: string
  pinned: boolean
}

export interface CanvasEdge {
  connection: Connection
  source: CanvasNode
  target: CanvasNode
  color: string
  width: number
}

export interface LayoutConfig {
  linkDistance: number
  chargeStrength: number
  centerX: number
  centerY: number
}
