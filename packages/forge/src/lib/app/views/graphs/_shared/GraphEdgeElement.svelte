<script lang="ts">
  /**
   * SVG edge element — path with arrowhead and optional label.
   */

  import type { LayoutEdge } from './graph-types';

  interface Props {
    edge: LayoutEdge;
    markerId: string;
  }

  let { edge, markerId }: Props = $props();

  /** Build SVG path from dagre edge points. */
  let pathD = $derived(() => {
    const pts = edge.points;
    if (pts.length === 0) return '';
    let d = `M ${pts[0].x} ${pts[0].y}`;
    if (pts.length === 2) {
      d += ` L ${pts[1].x} ${pts[1].y}`;
    } else if (pts.length >= 3) {
      // Use cubic bezier through control points
      for (let i = 1; i < pts.length - 1; i += 2) {
        const cp = pts[i];
        const end = pts[i + 1] ?? pts[pts.length - 1];
        d += ` Q ${cp.x} ${cp.y} ${end.x} ${end.y}`;
      }
    }
    return d;
  });

  /** Midpoint for label placement. */
  let labelPos = $derived(() => {
    const pts = edge.points;
    if (pts.length === 0) return { x: 0, y: 0 };
    const mid = Math.floor(pts.length / 2);
    return pts[mid];
  });

  let kindClass = $derived(`forge-graph-edge--${edge.kind}`);
</script>

<g class="forge-graph-edge {kindClass}">
  <path
    d={pathD()}
    class="forge-graph-edge__path"
    marker-end="url(#{markerId})"
    fill="none"
  />
  {#if edge.label}
    <text
      x={labelPos().x}
      y={labelPos().y - 6}
      text-anchor="middle"
      class="forge-graph-edge__label"
    >
      {edge.label}
    </text>
  {/if}
</g>

<style>
  .forge-graph-edge__path {
    stroke: var(--forge-graph-edge-default);
    stroke-width: 1.5;
    transition: stroke 0.15s ease;
  }

  .forge-graph-edge:hover .forge-graph-edge__path {
    stroke: var(--forge-accent-primary);
    stroke-width: 2;
  }

  .forge-graph-edge__label {
    fill: var(--forge-text-muted);
    font-size: 10px;
    font-family: var(--forge-font-family-ui, 'Outfit', sans-serif);
    pointer-events: none;
    user-select: none;
  }

  /* Kind variants */
  .forge-graph-edge--conditional .forge-graph-edge__path {
    stroke: var(--forge-graph-edge-conditional, #e6a817);
    stroke-dasharray: 6 3;
  }

  .forge-graph-edge--choice_sticky .forge-graph-edge__path {
    stroke: var(--forge-graph-edge-choice, #4caf50);
  }

  .forge-graph-edge--choice_oneshot .forge-graph-edge__path {
    stroke: var(--forge-graph-edge-choice, #4caf50);
    stroke-dasharray: 4 2;
  }

  .forge-graph-edge--terminal .forge-graph-edge__path {
    stroke: var(--forge-graph-edge-terminal, #888);
    stroke-dasharray: 2 2;
  }

  .forge-graph-edge--inheritance .forge-graph-edge__path {
    stroke: var(--forge-accent-secondary, #7b68ee);
  }

  .forge-graph-edge--containment .forge-graph-edge__path {
    stroke: var(--forge-text-muted);
    stroke-dasharray: 8 4;
  }

  .forge-graph-edge--reference .forge-graph-edge__path {
    stroke: var(--forge-text-muted);
    opacity: 0.6;
  }
</style>
