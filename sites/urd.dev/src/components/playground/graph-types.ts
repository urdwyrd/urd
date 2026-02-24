/** Shared types for graph visualisation components. */

/** A node in a graph visualisation. */
export interface GraphNode {
  id: string;
  label: string;
  kind: 'location' | 'section' | 'end';
  flag: 'start' | 'unreachable' | 'orphaned' | 'isolated' | null;
}

/** A directed edge in a graph visualisation. */
export interface GraphEdge {
  from: string;
  to: string;
  label: string;
  conditional: boolean;
}

/** A fully computed graph ready for rendering. */
export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}
