<script lang="ts">
  /**
   * WorldMap — 2D spatial location map using SVG.
   *
   * Reads urdJson locations and exits, placing locations on a grid based
   * on exit directions (north=up, south=down, east=right, west=left).
   * Starting location at centre. Lines for exits. Pan/zoom with pointer events.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { UrdWorld, UrdLocation } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface MapNode {
    id: string;
    name: string;
    x: number;
    y: number;
    isStart: boolean;
  }

  interface MapEdge {
    fromId: string;
    toId: string;
    fromX: number;
    fromY: number;
    toX: number;
    toY: number;
  }

  let nodes: MapNode[] = $state([]);
  let edges: MapEdge[] = $state([]);

  // Viewport transform
  let viewX = $state(0);
  let viewY = $state(0);
  let scale = $state(1);

  // Pan state
  let isPanning = $state(false);
  let panStartX = 0;
  let panStartY = 0;
  let panOriginX = 0;
  let panOriginY = 0;

  let svgElement: SVGSVGElement | undefined = $state(undefined);

  const CELL_SIZE = 140;
  const NODE_WIDTH = 120;
  const NODE_HEIGHT = 40;

  const DIRECTION_OFFSETS: Record<string, [number, number]> = {
    north: [0, -1],
    south: [0, 1],
    east: [1, 0],
    west: [-1, 0],
    northeast: [1, -1],
    northwest: [-1, -1],
    southeast: [1, 1],
    southwest: [-1, 1],
    up: [0, -1],
    down: [0, 1],
  };

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    buildMap();
    unsubscribers.push(bus.subscribe('compiler.completed', buildMap));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function buildMap(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    if (!urdJson || urdJson.locations.length === 0) {
      nodes = [];
      edges = [];
      return;
    }

    const positions = new Map<string, [number, number]>();
    const locationMap = new Map<string, UrdLocation>();
    for (const loc of urdJson.locations) {
      locationMap.set(loc.id, loc);
    }

    const startId = urdJson.world?.start ?? urdJson.locations[0]?.id;
    if (!startId) return;

    // BFS to assign grid positions based on exit directions
    positions.set(startId, [0, 0]);
    const queue: string[] = [startId];
    const visited = new Set<string>([startId]);

    while (queue.length > 0) {
      const currentId = queue.shift()!;
      const loc = locationMap.get(currentId);
      if (!loc) continue;

      const [cx, cy] = positions.get(currentId)!;

      for (const exit of loc.exits) {
        if (visited.has(exit.target)) continue;

        const offset = DIRECTION_OFFSETS[exit.direction.toLowerCase()] ?? [1, 0];
        let nx = cx + offset[0];
        let ny = cy + offset[1];

        // Avoid collisions by nudging
        const posKey = `${nx},${ny}`;
        let attempts = 0;
        while ([...positions.values()].some(([px, py]) => px === nx && py === ny) && attempts < 8) {
          nx += offset[0] || 1;
          ny += offset[1] || 1;
          attempts++;
        }

        positions.set(exit.target, [nx, ny]);
        visited.add(exit.target);
        queue.push(exit.target);
      }
    }

    // Place any unvisited locations in a row below
    let unvisitedCol = 0;
    const maxY = Math.max(0, ...[...positions.values()].map(([, y]) => y));
    for (const loc of urdJson.locations) {
      if (!positions.has(loc.id)) {
        positions.set(loc.id, [unvisitedCol, maxY + 2]);
        unvisitedCol++;
      }
    }

    // Build node list
    nodes = urdJson.locations
      .filter((loc) => positions.has(loc.id))
      .map((loc) => {
        const [gx, gy] = positions.get(loc.id)!;
        return {
          id: loc.id,
          name: loc.name || loc.id,
          x: gx * CELL_SIZE,
          y: gy * CELL_SIZE,
          isStart: loc.id === startId,
        };
      });

    // Build edge list
    const edgeList: MapEdge[] = [];
    for (const loc of urdJson.locations) {
      const fromPos = positions.get(loc.id);
      if (!fromPos) continue;

      for (const exit of loc.exits) {
        const toPos = positions.get(exit.target);
        if (!toPos) continue;

        edgeList.push({
          fromId: loc.id,
          toId: exit.target,
          fromX: fromPos[0] * CELL_SIZE,
          fromY: fromPos[1] * CELL_SIZE,
          toX: toPos[0] * CELL_SIZE,
          toY: toPos[1] * CELL_SIZE,
        });
      }
    }
    edges = edgeList;

    // Centre the viewport on the start node
    viewX = 0;
    viewY = 0;
    scale = 1;
  }

  function handlePointerDown(e: PointerEvent): void {
    if (e.button !== 0) return;
    isPanning = true;
    panStartX = e.clientX;
    panStartY = e.clientY;
    panOriginX = viewX;
    panOriginY = viewY;
    (e.target as Element)?.setPointerCapture?.(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent): void {
    if (!isPanning) return;
    viewX = panOriginX + (e.clientX - panStartX) / scale;
    viewY = panOriginY + (e.clientY - panStartY) / scale;
  }

  function handlePointerUp(): void {
    isPanning = false;
  }

  function handleWheel(e: WheelEvent): void {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    scale = Math.max(0.2, Math.min(3, scale * factor));
  }
</script>

<div class="forge-worldmap">
  {#if nodes.length === 0}
    <div class="forge-worldmap__empty">
      <p>No location data available</p>
      <p class="forge-worldmap__hint">Compile a project with locations to see the world map</p>
    </div>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <svg
      class="forge-worldmap__svg"
      bind:this={svgElement}
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onwheel={handleWheel}
    >
      <g transform="translate({viewX * scale + 400}, {viewY * scale + 300}) scale({scale})">
        <!-- Edges -->
        {#each edges as edge}
          <line
            class="forge-worldmap__edge"
            x1={edge.fromX}
            y1={edge.fromY}
            x2={edge.toX}
            y2={edge.toY}
          />
        {/each}

        <!-- Nodes -->
        {#each nodes as node}
          <g class="forge-worldmap__node-group">
            <rect
              class="forge-worldmap__node-rect"
              class:forge-worldmap__node-rect--start={node.isStart}
              x={node.x - NODE_WIDTH / 2}
              y={node.y - NODE_HEIGHT / 2}
              width={NODE_WIDTH}
              height={NODE_HEIGHT}
              rx="4"
            />
            <text
              class="forge-worldmap__node-label"
              x={node.x}
              y={node.y + 4}
              text-anchor="middle"
            >
              {node.name.length > 14 ? node.name.slice(0, 12) + '...' : node.name}
            </text>
          </g>
        {/each}
      </g>
    </svg>

    <div class="forge-worldmap__controls">
      <span class="forge-worldmap__zoom-label">{Math.round(scale * 100)}%</span>
    </div>
  {/if}
</div>

<style>
  .forge-worldmap {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    position: relative;
  }

  .forge-worldmap__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-worldmap__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-worldmap__svg {
    width: 100%;
    height: 100%;
    cursor: grab;
    user-select: none;
  }

  .forge-worldmap__svg:active {
    cursor: grabbing;
  }

  .forge-worldmap__edge {
    stroke: var(--forge-border-zone);
    stroke-width: 1.5;
    opacity: 0.6;
  }

  .forge-worldmap__node-rect {
    fill: var(--forge-bg-secondary);
    stroke: var(--forge-border-zone);
    stroke-width: 1.5;
    cursor: pointer;
  }

  .forge-worldmap__node-rect:hover {
    fill: var(--forge-bg-hover);
  }

  .forge-worldmap__node-rect--start {
    stroke: var(--forge-runtime-play-active, #4caf50);
    stroke-width: 2;
  }

  .forge-worldmap__node-label {
    fill: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: 11px;
    pointer-events: none;
  }

  .forge-worldmap__controls {
    position: absolute;
    bottom: var(--forge-space-md);
    right: var(--forge-space-md);
    padding: 2px 8px;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
  }

  .forge-worldmap__zoom-label {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }
</style>
