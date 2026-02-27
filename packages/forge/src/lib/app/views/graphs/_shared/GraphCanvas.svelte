<script lang="ts">
  /**
   * Shared graph canvas — dagre layout + SVG rendering + pan/zoom.
   * All graph views pass ForgeGraphData here; GraphCanvas handles layout and interaction.
   */

  import dagre from '@dagrejs/dagre';
  import type { ForgeGraphData, GraphLayout, LayoutNode, LayoutEdge } from './graph-types';
  import GraphNodeElement from './GraphNodeElement.svelte';
  import GraphEdgeElement from './GraphEdgeElement.svelte';

  interface Props {
    data: ForgeGraphData;
    rankdir?: 'TB' | 'LR' | 'BT' | 'RL';
    nodesep?: number;
    ranksep?: number;
    nodeWidth?: number;
    nodeHeight?: number;
    onNodeClick?: (nodeId: string) => void;
    onNodeDblClick?: (nodeId: string) => void;
    emptyMessage?: string;
  }

  let {
    data,
    rankdir = 'TB',
    nodesep = 60,
    ranksep = 80,
    nodeWidth = 150,
    nodeHeight = 40,
    onNodeClick,
    onNodeDblClick,
    emptyMessage = 'No data to display',
  }: Props = $props();

  // Pan/zoom state
  let viewX = $state(0);
  let viewY = $state(0);
  let viewScale = $state(1);

  // Drag state
  let dragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartViewX = 0;
  let dragStartViewY = 0;

  // Container ref
  let svgElement: SVGSVGElement | undefined = $state();

  // Unique marker ID for this instance
  const markerId = `arrow-${Math.random().toString(36).slice(2, 8)}`;

  // Compute layout whenever data or layout params change
  let layout = $derived(computeLayout(data, rankdir, nodesep, ranksep, nodeWidth, nodeHeight));

  function computeLayout(
    graphData: ForgeGraphData,
    dir: string,
    nsep: number,
    rsep: number,
    nw: number,
    nh: number,
  ): GraphLayout | null {
    if (!graphData.nodes.length) return null;

    const g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: dir, nodesep: nsep, ranksep: rsep, marginx: 20, marginy: 20 });
    g.setDefaultEdgeLabel(() => ({}));

    for (const node of graphData.nodes) {
      const w = node.kind === 'terminal' ? nw * 0.6 : nw;
      const h = node.kind === 'terminal' ? nh * 0.7 : nh;
      g.setNode(node.id, { width: w, height: h });
    }

    for (const edge of graphData.edges) {
      g.setEdge(edge.source, edge.target, { label: edge.id });
    }

    dagre.layout(g);

    const layoutNodes: LayoutNode[] = graphData.nodes.map((node) => {
      const dagNode = g.node(node.id);
      return {
        ...node,
        x: dagNode.x,
        y: dagNode.y,
        width: dagNode.width,
        height: dagNode.height,
      };
    });

    const layoutEdges: LayoutEdge[] = graphData.edges.map((edge) => {
      const dagEdge = g.edge(edge.source, edge.target);
      return {
        ...edge,
        points: dagEdge?.points ?? [],
      };
    });

    const graphMeta = g.graph();
    return {
      nodes: layoutNodes,
      edges: layoutEdges,
      width: graphMeta.width ?? 0,
      height: graphMeta.height ?? 0,
    };
  }

  // Fit to view on initial layout or data change
  $effect(() => {
    if (layout && svgElement) {
      fitToView();
    }
  });

  function fitToView() {
    if (!layout || !svgElement) return;
    const rect = svgElement.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const padding = 40;
    const scaleX = (rect.width - padding * 2) / layout.width;
    const scaleY = (rect.height - padding * 2) / layout.height;
    const newScale = Math.min(scaleX, scaleY, 2);

    viewScale = Math.max(0.2, newScale);
    viewX = (rect.width - layout.width * viewScale) / 2;
    viewY = (rect.height - layout.height * viewScale) / 2;
  }

  // Pan handlers
  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    // Only start drag if clicking on background (not a node)
    const target = e.target as Element;
    if (target.closest('.forge-graph-node')) return;

    dragging = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    dragStartViewX = viewX;
    dragStartViewY = viewY;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!dragging) return;
    viewX = dragStartViewX + (e.clientX - dragStartX);
    viewY = dragStartViewY + (e.clientY - dragStartY);
  }

  function handlePointerUp() {
    dragging = false;
  }

  // Zoom handler
  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const rect = svgElement!.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const oldScale = viewScale;
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.2, Math.min(4, viewScale * delta));

    // Zoom towards mouse position
    viewX = mouseX - (mouseX - viewX) * (newScale / oldScale);
    viewY = mouseY - (mouseY - viewY) * (newScale / oldScale);
    viewScale = newScale;
  }
</script>

<div class="forge-graph-canvas">
  {#if !layout}
    <div class="forge-graph-canvas__empty">
      <p>{emptyMessage}</p>
    </div>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <svg
      bind:this={svgElement}
      class="forge-graph-canvas__svg"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onwheel={handleWheel}
    >
      <defs>
        <marker
          id={markerId}
          viewBox="0 0 10 10"
          refX="10"
          refY="5"
          markerWidth="8"
          markerHeight="8"
          orient="auto-start-reverse"
        >
          <path d="M 0 0 L 10 5 L 0 10 Z" fill="var(--forge-graph-edge-default)" />
        </marker>
      </defs>

      <g transform="translate({viewX}, {viewY}) scale({viewScale})">
        {#each layout.edges as edge (edge.id)}
          <GraphEdgeElement {edge} {markerId} />
        {/each}
        {#each layout.nodes as node (node.id)}
          <GraphNodeElement
            {node}
            onclick={onNodeClick}
            ondblclick={onNodeDblClick}
          />
        {/each}
      </g>
    </svg>

    <div class="forge-graph-canvas__toolbar">
      <button
        class="forge-graph-canvas__btn"
        onclick={fitToView}
        title="Fit to view"
      >⊡</button>
      <button
        class="forge-graph-canvas__btn"
        onclick={() => { viewScale = Math.min(4, viewScale * 1.2); }}
        title="Zoom in"
      >+</button>
      <button
        class="forge-graph-canvas__btn"
        onclick={() => { viewScale = Math.max(0.2, viewScale * 0.8); }}
        title="Zoom out"
      >−</button>
    </div>
  {/if}
</div>

<style>
  .forge-graph-canvas {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: var(--forge-bg-primary);
  }

  .forge-graph-canvas__svg {
    width: 100%;
    height: 100%;
    display: block;
    cursor: grab;
  }

  .forge-graph-canvas__svg:active {
    cursor: grabbing;
  }

  .forge-graph-canvas__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
  }

  .forge-graph-canvas__toolbar {
    position: absolute;
    bottom: 8px;
    right: 8px;
    display: flex;
    gap: 2px;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    padding: 2px;
  }

  .forge-graph-canvas__btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 14px;
    cursor: pointer;
  }

  .forge-graph-canvas__btn:hover {
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }
</style>
