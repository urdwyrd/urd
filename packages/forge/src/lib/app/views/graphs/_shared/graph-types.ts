/**
 * Shared graph data model for all Forge graph views.
 * Consumed by GraphCanvas.svelte — views transform projection data into this shape.
 */

export type GraphNodeKind =
  | 'location'
  | 'section'
  | 'terminal'
  | 'type'
  | 'property'
  | 'entity'
  | 'file';

export interface GraphNodeFlags {
  start?: boolean;
  unreachable?: boolean;
  isolated?: boolean;
  orphaned?: boolean;
  selected?: boolean;
}

export interface ForgeGraphNode {
  id: string;
  label: string;
  kind: GraphNodeKind;
  flags?: GraphNodeFlags;
  metadata?: Record<string, unknown>;
}

export type GraphEdgeKind =
  | 'normal'
  | 'conditional'
  | 'choice_sticky'
  | 'choice_oneshot'
  | 'terminal'
  | 'inheritance'
  | 'containment'
  | 'reference';

export interface ForgeGraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  kind: GraphEdgeKind;
  metadata?: Record<string, unknown>;
}

export interface ForgeGraphData {
  nodes: ForgeGraphNode[];
  edges: ForgeGraphEdge[];
}

/** Positioned node after dagre layout. */
export interface LayoutNode extends ForgeGraphNode {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Positioned edge after dagre layout. */
export interface LayoutEdge extends ForgeGraphEdge {
  points: Array<{ x: number; y: number }>;
}

/** Full layout result from dagre. */
export interface GraphLayout {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  width: number;
  height: number;
}
