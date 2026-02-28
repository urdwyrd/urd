<script lang="ts">
  /**
   * ForceCanvas — Canvas 2D renderer with d3-force simulation.
   * Drop-in alternative to GraphCanvas.svelte for force-directed layouts.
   */

  import { onMount, onDestroy } from 'svelte';
  import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide } from 'd3-force';
  import type { Simulation, SimulationNodeDatum, SimulationLinkDatum } from 'd3-force';
  import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge, GraphNodeKind, GraphEdgeKind } from './graph-types';
  import { readGraphTheme, type GraphTheme } from './force-theme';
  import { bus } from '$lib/framework/bus/MessageBus';

  interface Props {
    data: ForgeGraphData;
    onNodeClick?: (nodeId: string) => void;
    onNodeDblClick?: (nodeId: string) => void;
    emptyMessage?: string;
  }

  let { data, onNodeClick, onNodeDblClick, emptyMessage = 'No data to display' }: Props = $props();

  // Canvas and container refs
  let containerEl: HTMLDivElement | undefined = $state();
  let canvasEl: HTMLCanvasElement | undefined = $state();
  let ctx: CanvasRenderingContext2D | null = null;

  // Camera state
  let viewX = 0;
  let viewY = 0;
  let viewScale = 1;

  // Interaction state
  let dragging = false;
  let dragTarget: SimNode | null = null;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartViewX = 0;
  let dragStartViewY = 0;
  let pointerDownTime = 0;
  let pointerMoved = false;
  let hoveredNode: SimNode | null = null;
  let simulationPaused = $state(false);

  // Theme
  let theme: GraphTheme;

  // Simulation types
  interface SimNode extends SimulationNodeDatum, ForgeGraphNode {
    x: number;
    y: number;
    radius: number;
  }

  interface SimEdge extends SimulationLinkDatum<SimNode> {
    id: string;
    kind: GraphEdgeKind;
    label?: string;
  }

  let simNodes: SimNode[] = [];
  let simEdges: SimEdge[] = [];
  let simulation: Simulation<SimNode, SimEdge> | null = null;
  let rafId: number | null = null;
  let canvasWidth = 0;
  let canvasHeight = 0;

  // Node sizes by kind
  const NODE_RADII: Record<GraphNodeKind, number> = {
    location: 14,
    section: 12,
    terminal: 8,
    type: 14,
    property: 9,
    entity: 12,
    file: 16,
  };

  // Edge colours by kind (resolved at render time from theme)
  function edgeColour(kind: GraphEdgeKind): string {
    switch (kind) {
      case 'conditional': return theme.edgeConditional;
      case 'choice_sticky': return theme.edgeChoice;
      case 'choice_oneshot': return theme.edgeChoice;
      case 'terminal': return theme.edgeTerminal;
      case 'inheritance': return theme.accentSecondary;
      case 'containment': return theme.textMuted;
      case 'reference': return theme.textMuted;
      default: return theme.edgeDefault;
    }
  }

  function edgeDash(kind: GraphEdgeKind): number[] {
    switch (kind) {
      case 'conditional': return [6, 3];
      case 'choice_oneshot': return [4, 2];
      case 'terminal': return [2, 2];
      case 'containment': return [8, 4];
      default: return [];
    }
  }

  function edgeWidth(kind: GraphEdgeKind): number {
    switch (kind) {
      case 'choice_sticky': return 2.5;
      case 'inheritance': return 2;
      default: return 1.2;
    }
  }

  function nodeColour(node: SimNode): string {
    if (node.flags?.start) return theme.nodeStart;
    if (node.flags?.unreachable) return theme.nodeUnreachable;
    if (node.flags?.isolated) return theme.nodeIsolated;
    if (node.flags?.orphaned) return theme.nodeIsolated;
    return theme.nodeDefault;
  }

  function nodeBorderColour(node: SimNode): string {
    if (node === hoveredNode) return theme.accentPrimary;
    if (node.flags?.selected) return theme.nodeSelected;
    if (node.flags?.start) return theme.nodeStart;
    if (node.flags?.unreachable) return theme.nodeUnreachable;
    return theme.borderZone;
  }

  // --- Simulation ---

  function buildSimulation() {
    if (simulation) {
      simulation.stop();
    }

    // Build sim nodes with radii
    simNodes = data.nodes.map((n) => ({
      ...n,
      x: (Math.random() - 0.5) * 300,
      y: (Math.random() - 0.5) * 300,
      radius: NODE_RADII[n.kind] ?? 10,
    }));

    const nodeMap = new Map(simNodes.map((n) => [n.id, n]));

    // Build sim edges (only edges where both source and target exist)
    simEdges = data.edges
      .filter((e) => nodeMap.has(e.source) && nodeMap.has(e.target))
      .map((e) => ({
        source: e.source,
        target: e.target,
        id: e.id,
        kind: e.kind,
        label: e.label,
      }));

    simulation = forceSimulation<SimNode>(simNodes)
      .force(
        'link',
        forceLink<SimNode, SimEdge>(simEdges)
          .id((d) => d.id)
          .distance(80)
          .strength(0.4),
      )
      .force('charge', forceManyBody<SimNode>().strength(-250).distanceMax(400))
      .force('center', forceCenter(0, 0).strength(0.05))
      .force('collide', forceCollide<SimNode>((d) => d.radius + 6).iterations(2))
      .alphaDecay(0.02)
      .on('tick', () => {
        /* RAF loop handles drawing */
      });

    // Reset camera
    viewX = canvasWidth / 2;
    viewY = canvasHeight / 2;
    viewScale = 1;
    simulationPaused = false;
  }

  // --- Rendering ---

  function draw() {
    if (!ctx || !canvasEl) return;

    const dpr = window.devicePixelRatio || 1;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, canvasWidth, canvasHeight);

    ctx.save();
    ctx.translate(viewX, viewY);
    ctx.scale(viewScale, viewScale);

    // Draw edges
    for (const edge of simEdges) {
      const source = edge.source as SimNode;
      const target = edge.target as SimNode;
      if (source.x == null || target.x == null) continue;

      ctx.beginPath();
      ctx.moveTo(source.x, source.y);
      ctx.lineTo(target.x, target.y);

      ctx.strokeStyle = edgeColour(edge.kind);
      ctx.lineWidth = edgeWidth(edge.kind) / viewScale;
      const dash = edgeDash(edge.kind);
      ctx.setLineDash(dash.length ? dash.map((d) => d / viewScale) : []);
      ctx.stroke();
      ctx.setLineDash([]);

      // Arrowhead
      drawArrowhead(ctx, source.x, source.y, target.x, target.y, (target as SimNode).radius);
    }

    // Draw nodes
    for (const node of simNodes) {
      if (node.x == null) continue;

      const r = node.radius;
      const isHovered = node === hoveredNode;

      // Shadow for hovered
      if (isHovered) {
        ctx.shadowColor = theme.accentPrimary;
        ctx.shadowBlur = 12 / viewScale;
      }

      ctx.beginPath();
      if (node.kind === 'terminal') {
        ctx.arc(node.x, node.y, r, 0, Math.PI * 2);
      } else if (node.kind === 'type') {
        // Diamond
        ctx.moveTo(node.x, node.y - r);
        ctx.lineTo(node.x + r, node.y);
        ctx.lineTo(node.x, node.y + r);
        ctx.lineTo(node.x - r, node.y);
        ctx.closePath();
      } else {
        // Rounded rect
        roundRect(ctx, node.x - r, node.y - r * 0.7, r * 2, r * 1.4, 4);
      }

      ctx.fillStyle = nodeColour(node);
      ctx.fill();
      ctx.strokeStyle = nodeBorderColour(node);
      ctx.lineWidth = (isHovered ? 2.5 : 1.5) / viewScale;

      if (node.flags?.unreachable || node.flags?.isolated || node.flags?.orphaned) {
        ctx.setLineDash([4 / viewScale, 2 / viewScale]);
      }
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.shadowColor = 'transparent';
      ctx.shadowBlur = 0;

      // Label — only when zoomed in enough
      if (viewScale > 0.35) {
        const fontSize = Math.max(9, Math.min(12, 11 / viewScale));
        ctx.font = `${fontSize}px 'Outfit', sans-serif`;
        ctx.fillStyle = theme.textPrimary;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';

        const label = truncateLabel(node.label, r * 2);
        ctx.fillText(label, node.x, node.y);
      }
    }

    ctx.restore();

    // Edge labels at higher zoom
    if (viewScale > 0.7) {
      ctx.save();
      ctx.translate(viewX, viewY);
      ctx.scale(viewScale, viewScale);
      const fontSize = Math.max(8, Math.min(10, 9 / viewScale));
      ctx.font = `${fontSize}px 'Outfit', sans-serif`;
      ctx.fillStyle = theme.textMuted;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'bottom';
      for (const edge of simEdges) {
        if (!edge.label) continue;
        const source = edge.source as SimNode;
        const target = edge.target as SimNode;
        if (source.x == null || target.x == null) continue;
        const mx = (source.x + target.x) / 2;
        const my = (source.y + target.y) / 2 - 4;
        ctx.fillText(edge.label, mx, my);
      }
      ctx.restore();
    }
  }

  function drawArrowhead(
    c: CanvasRenderingContext2D,
    x1: number, y1: number,
    x2: number, y2: number,
    targetRadius: number,
  ) {
    const dx = x2 - x1;
    const dy = y2 - y1;
    const len = Math.sqrt(dx * dx + dy * dy);
    if (len < 1) return;

    const ux = dx / len;
    const uy = dy / len;
    const tipX = x2 - ux * targetRadius;
    const tipY = y2 - uy * targetRadius;
    const sz = 6 / viewScale;

    c.beginPath();
    c.moveTo(tipX, tipY);
    c.lineTo(tipX - ux * sz - uy * sz * 0.5, tipY - uy * sz + ux * sz * 0.5);
    c.lineTo(tipX - ux * sz + uy * sz * 0.5, tipY - uy * sz - ux * sz * 0.5);
    c.closePath();
    c.fill();
  }

  function roundRect(
    c: CanvasRenderingContext2D,
    x: number, y: number, w: number, h: number, r: number,
  ) {
    c.moveTo(x + r, y);
    c.lineTo(x + w - r, y);
    c.quadraticCurveTo(x + w, y, x + w, y + r);
    c.lineTo(x + w, y + h - r);
    c.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    c.lineTo(x + r, y + h);
    c.quadraticCurveTo(x, y + h, x, y + h - r);
    c.lineTo(x, y + r);
    c.quadraticCurveTo(x, y, x + r, y);
    c.closePath();
  }

  function truncateLabel(label: string, maxWidth: number): string {
    if (label.length <= 14) return label;
    return label.slice(0, 12) + '…';
  }

  // --- Hit testing ---

  function screenToWorld(sx: number, sy: number): { x: number; y: number } {
    return {
      x: (sx - viewX) / viewScale,
      y: (sy - viewY) / viewScale,
    };
  }

  function findNodeAt(sx: number, sy: number): SimNode | null {
    const { x, y } = screenToWorld(sx, sy);
    let closest: SimNode | null = null;
    let closestDist = Infinity;
    for (const node of simNodes) {
      if (node.x == null) continue;
      const dx = node.x - x;
      const dy = node.y - y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < node.radius + 4 && dist < closestDist) {
        closest = node;
        closestDist = dist;
      }
    }
    return closest;
  }

  // --- Interaction ---

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0 || !canvasEl) return;

    const rect = canvasEl.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;

    pointerDownTime = Date.now();
    pointerMoved = false;

    const hit = findNodeAt(sx, sy);
    if (hit) {
      dragTarget = hit;
      hit.fx = hit.x;
      hit.fy = hit.y;
      if (simulation && simulation.alpha() < 0.1) {
        simulation.alpha(0.3).restart();
      }
    } else {
      dragTarget = null;
      dragStartX = e.clientX;
      dragStartY = e.clientY;
      dragStartViewX = viewX;
      dragStartViewY = viewY;
    }

    dragging = true;
    canvasEl.setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;

    if (dragging) {
      pointerMoved = true;
      if (dragTarget) {
        // Drag node
        const { x, y } = screenToWorld(sx, sy);
        dragTarget.fx = x;
        dragTarget.fy = y;
        if (simulation && simulation.alpha() < 0.05) {
          simulation.alpha(0.1).restart();
        }
      } else {
        // Pan camera
        viewX = dragStartViewX + (e.clientX - dragStartX);
        viewY = dragStartViewY + (e.clientY - dragStartY);
      }
    } else {
      // Hover
      hoveredNode = findNodeAt(sx, sy);
      if (canvasEl) {
        canvasEl.style.cursor = hoveredNode ? 'pointer' : 'grab';
      }
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!dragging) return;

    if (dragTarget) {
      // Release node — unpin unless shift held
      if (!e.shiftKey) {
        dragTarget.fx = null;
        dragTarget.fy = null;
      }

      // Click detection (short press, no movement)
      if (!pointerMoved || Date.now() - pointerDownTime < 200) {
        onNodeClick?.(dragTarget.id);
      }

      dragTarget = null;
    }

    dragging = false;
  }

  function handleDblClick(e: MouseEvent) {
    if (!canvasEl || !onNodeDblClick) return;
    const rect = canvasEl.getBoundingClientRect();
    const hit = findNodeAt(e.clientX - rect.left, e.clientY - rect.top);
    if (hit) onNodeDblClick(hit.id);
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (!canvasEl) return;

    const rect = canvasEl.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    const oldScale = viewScale;
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.05, Math.min(12, viewScale * delta));

    viewX = mx - (mx - viewX) * (newScale / oldScale);
    viewY = my - (my - viewY) * (newScale / oldScale);
    viewScale = newScale;
  }

  // --- Toolbar ---

  function fitToView() {
    if (!simNodes.length) return;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const n of simNodes) {
      if (n.x == null) continue;
      minX = Math.min(minX, n.x - n.radius);
      minY = Math.min(minY, n.y - n.radius);
      maxX = Math.max(maxX, n.x + n.radius);
      maxY = Math.max(maxY, n.y + n.radius);
    }

    const graphW = maxX - minX || 1;
    const graphH = maxY - minY || 1;
    const pad = 60;
    const scaleX = (canvasWidth - pad * 2) / graphW;
    const scaleY = (canvasHeight - pad * 2) / graphH;
    viewScale = Math.min(scaleX, scaleY, 2);
    viewX = canvasWidth / 2 - ((minX + maxX) / 2) * viewScale;
    viewY = canvasHeight / 2 - ((minY + maxY) / 2) * viewScale;
  }

  function togglePause() {
    if (!simulation) return;
    if (simulationPaused) {
      simulation.alpha(0.3).restart();
      simulationPaused = false;
    } else {
      simulation.stop();
      simulationPaused = true;
    }
  }

  // --- Lifecycle ---

  function resizeCanvas() {
    if (!containerEl || !canvasEl) return;
    const rect = containerEl.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvasWidth = rect.width;
    canvasHeight = rect.height;
    canvasEl.width = rect.width * dpr;
    canvasEl.height = rect.height * dpr;
    canvasEl.style.width = `${rect.width}px`;
    canvasEl.style.height = `${rect.height}px`;
  }

  function animationLoop() {
    draw();
    rafId = requestAnimationFrame(animationLoop);
  }

  let resizeObserver: ResizeObserver | null = null;

  let unsubTheme: (() => void) | null = null;

  onMount(() => {
    if (!canvasEl || !containerEl) return;
    ctx = canvasEl.getContext('2d');
    theme = readGraphTheme();
    resizeCanvas();

    canvasEl.addEventListener('wheel', handleWheel, { passive: false });

    resizeObserver = new ResizeObserver(() => {
      resizeCanvas();
    });
    resizeObserver.observe(containerEl);

    buildSimulation();
    rafId = requestAnimationFrame(animationLoop);

    unsubTheme = bus.subscribe('theme.changed', () => {
      theme = readGraphTheme();
    });
  });

  // Rebuild simulation when data changes
  $effect(() => {
    // Touch data to track dependency
    const _nodes = data.nodes;
    const _edges = data.edges;
    if (ctx && canvasEl) {
      buildSimulation();
    }
  });

  onDestroy(() => {
    if (rafId != null) cancelAnimationFrame(rafId);
    if (simulation) simulation.stop();
    if (resizeObserver) resizeObserver.disconnect();
    if (canvasEl) canvasEl.removeEventListener('wheel', handleWheel);
    unsubTheme?.();
  });
</script>

<div class="forge-force-canvas" bind:this={containerEl}>
  {#if data.nodes.length === 0}
    <div class="forge-force-canvas__empty">
      <p>{emptyMessage}</p>
    </div>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <canvas
      bind:this={canvasEl}
      class="forge-force-canvas__canvas"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      ondblclick={handleDblClick}
    ></canvas>

    <div class="forge-force-canvas__toolbar">
      <button class="forge-force-canvas__btn" onclick={fitToView} title="Fit to view">⊡</button>
      <button class="forge-force-canvas__btn" onclick={() => { viewScale = Math.min(12, viewScale * 1.2); }} title="Zoom in">+</button>
      <button class="forge-force-canvas__btn" onclick={() => { viewScale = Math.max(0.05, viewScale * 0.8); }} title="Zoom out">−</button>
      <button class="forge-force-canvas__btn" onclick={togglePause} title={simulationPaused ? 'Resume' : 'Pause'}>
        {simulationPaused ? '▶' : '⏸'}
      </button>
    </div>
  {/if}
</div>

<style>
  .forge-force-canvas {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: var(--forge-bg-primary);
  }

  .forge-force-canvas__canvas {
    display: block;
    cursor: grab;
  }

  .forge-force-canvas__canvas:active {
    cursor: grabbing;
  }

  .forge-force-canvas__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
  }

  .forge-force-canvas__toolbar {
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

  .forge-force-canvas__btn {
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

  .forge-force-canvas__btn:hover {
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }
</style>
