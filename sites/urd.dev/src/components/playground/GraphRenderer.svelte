<script lang="ts">
  import { onMount } from 'svelte';
  import type { GraphData, GraphNode } from './graph-types';

  interface Props {
    graph: GraphData;
    direction?: 'TB' | 'LR';
    onNodeClick?: (nodeId: string) => void;
  }

  let { graph, direction = 'TB', onNodeClick }: Props = $props();

  // Layout state
  let svgEl: SVGSVGElement | undefined = $state();
  let layoutNodes: Array<{ id: string; label: string; kind: string; flag: string | null; x: number; y: number; w: number; h: number }> = $state([]);
  let layoutEdges: Array<{ from: string; to: string; label: string; conditional: boolean; points: Array<{ x: number; y: number }> }> = $state([]);

  // Pan/zoom
  let tx = $state(0);
  let ty = $state(0);
  let scale = $state(1);
  let isPanning = false;
  let panStartX = 0;
  let panStartY = 0;

  // Node sizing
  const NODE_H = 32;
  const CHAR_W = 7.5;
  const NODE_PAD = 24;
  const MIN_W = 80;

  // Re-layout when graph or direction changes.
  $effect(() => {
    void doLayout(graph, direction);
  });

  async function doLayout(g: GraphData, dir: string) {
    if (g.nodes.length === 0) {
      layoutNodes = [];
      layoutEdges = [];
      return;
    }

    const dagre = await import('@dagrejs/dagre');
    const dg = new dagre.graphlib.Graph();
    dg.setGraph({ rankdir: dir, nodesep: 40, ranksep: 60, marginx: 20, marginy: 20 });
    dg.setDefaultEdgeLabel(() => ({}));

    for (const node of g.nodes) {
      const w = Math.max(node.label.length * CHAR_W + NODE_PAD, MIN_W);
      dg.setNode(node.id, { width: w, height: NODE_H });
    }

    for (const edge of g.edges) {
      dg.setEdge(edge.from, edge.to);
    }

    dagre.layout(dg);

    layoutNodes = g.nodes.map((node) => {
      const pos = dg.node(node.id);
      return {
        id: node.id,
        label: node.label,
        kind: node.kind,
        flag: node.flag,
        x: pos.x,
        y: pos.y,
        w: pos.width,
        h: pos.height,
      };
    });

    layoutEdges = g.edges.map((edge) => {
      const de = dg.edge(edge.from, edge.to);
      return {
        from: edge.from,
        to: edge.to,
        label: edge.label,
        conditional: edge.conditional,
        points: de?.points ?? [],
      };
    });

    fitToView();
  }

  function fitToView() {
    if (!svgEl || layoutNodes.length === 0) return;
    const rect = svgEl.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const n of layoutNodes) {
      minX = Math.min(minX, n.x - n.w / 2);
      minY = Math.min(minY, n.y - n.h / 2);
      maxX = Math.max(maxX, n.x + n.w / 2);
      maxY = Math.max(maxY, n.y + n.h / 2);
    }

    const gw = maxX - minX + 40;
    const gh = maxY - minY + 40;
    const fitScale = Math.min(rect.width / gw, rect.height / gh, 1.5);

    scale = fitScale;
    tx = (rect.width - gw * fitScale) / 2 - minX * fitScale + 20 * fitScale;
    ty = (rect.height - gh * fitScale) / 2 - minY * fitScale + 20 * fitScale;
  }

  // Pan handlers
  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    isPanning = true;
    panStartX = e.clientX - tx;
    panStartY = e.clientY - ty;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!isPanning) return;
    tx = e.clientX - panStartX;
    ty = e.clientY - panStartY;
  }

  function onPointerUp() {
    isPanning = false;
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    if (!svgEl) return;
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.2, Math.min(3, scale * delta));
    const rect = svgEl.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    tx = cx - (cx - tx) * (newScale / scale);
    ty = cy - (cy - ty) * (newScale / scale);
    scale = newScale;
  }

  // SVG helpers
  function edgePath(points: Array<{ x: number; y: number }>): string {
    if (points.length === 0) return '';
    let d = `M ${points[0].x} ${points[0].y}`;
    for (let i = 1; i < points.length; i++) {
      d += ` L ${points[i].x} ${points[i].y}`;
    }
    return d;
  }

  function nodeStroke(kind: string, flag: string | null): string {
    if (flag === 'unreachable') return 'var(--rose)';
    if (flag === 'orphaned') return 'var(--amber-light)';
    if (kind === 'end') return 'var(--faint)';
    return 'var(--gold-dim)';
  }

  function nodeFill(kind: string, flag: string | null): string {
    if (flag === 'unreachable') return 'color-mix(in srgb, var(--rose) 12%, transparent)';
    if (flag === 'orphaned') return 'color-mix(in srgb, var(--amber-light) 12%, transparent)';
    if (kind === 'end') return 'var(--raise)';
    return 'var(--surface)';
  }

  function strokeDash(flag: string | null): string {
    if (flag === 'unreachable') return '4 3';
    return 'none';
  }
</script>

<div class="graph-container">
  <svg
    bind:this={svgEl}
    class="graph-svg"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onwheel={onWheel}
  >
    <defs>
      <marker
        id="arrow"
        markerWidth="8"
        markerHeight="6"
        refX="8"
        refY="3"
        orient="auto"
      >
        <polygon points="0 0, 8 3, 0 6" fill="var(--dim)" />
      </marker>
      <marker
        id="arrow-cond"
        markerWidth="8"
        markerHeight="6"
        refX="8"
        refY="3"
        orient="auto"
      >
        <polygon points="0 0, 8 3, 0 6" fill="var(--amber-light)" />
      </marker>
    </defs>

    <g transform="translate({tx}, {ty}) scale({scale})">
      {#each layoutEdges as edge}
        <path
          d={edgePath(edge.points)}
          fill="none"
          stroke={edge.conditional ? 'var(--amber-light)' : 'var(--dim)'}
          stroke-width="1.5"
          stroke-dasharray={edge.conditional ? '4 3' : 'none'}
          marker-end={edge.conditional ? 'url(#arrow-cond)' : 'url(#arrow)'}
        />
        {#if edge.label}
          {@const mid = edge.points[Math.floor(edge.points.length / 2)]}
          {#if mid}
            <text
              x={mid.x}
              y={mid.y - 6}
              text-anchor="middle"
              class="edge-label"
            >{edge.label}</text>
          {/if}
        {/if}
      {/each}

      {#each layoutNodes as node}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <g
          class="graph-node"
          class:clickable={!!onNodeClick}
          onclick={() => onNodeClick?.(node.id)}
          onkeydown={(e) => { if (e.key === 'Enter') onNodeClick?.(node.id); }}
          role={onNodeClick ? 'button' : undefined}
          tabindex={onNodeClick ? 0 : undefined}
        >
          <rect
            x={node.x - node.w / 2}
            y={node.y - node.h / 2}
            width={node.w}
            height={node.h}
            rx={node.kind === 'end' ? 12 : 4}
            fill={nodeFill(node.kind, node.flag)}
            stroke={nodeStroke(node.kind, node.flag)}
            stroke-width="1"
            stroke-dasharray={strokeDash(node.flag)}
          />
          <text
            x={node.x}
            y={node.y + 4}
            text-anchor="middle"
            class="node-label"
          >{node.label}</text>
        </g>
      {/each}
    </g>
  </svg>

  {#if graph.nodes.length === 0}
    <div class="graph-empty">No graph data available.</div>
  {/if}
</div>

<style>
  .graph-container {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 200px;
    overflow: hidden;
  }

  .graph-svg {
    width: 100%;
    height: 100%;
    cursor: grab;
    touch-action: none;
  }

  .graph-svg:active {
    cursor: grabbing;
  }

  .node-label {
    font-family: var(--mono);
    font-size: 11px;
    fill: var(--text);
    pointer-events: none;
    user-select: none;
  }

  .edge-label {
    font-family: var(--mono);
    font-size: 9px;
    fill: var(--faint);
    pointer-events: none;
    user-select: none;
  }

  .graph-node.clickable {
    cursor: pointer;
  }

  .graph-node.clickable:hover rect {
    stroke-width: 2;
  }

  .graph-node:focus-visible rect {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .graph-empty {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--body);
    font-size: 13px;
    color: var(--faint);
  }
</style>
