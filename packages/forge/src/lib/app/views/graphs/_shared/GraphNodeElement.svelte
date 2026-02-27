<script lang="ts">
  /**
   * SVG node element — rounded rectangle with label and flag-based styling.
   */

  import type { LayoutNode } from './graph-types';

  interface Props {
    node: LayoutNode;
    onclick?: (nodeId: string) => void;
    ondblclick?: (nodeId: string) => void;
  }

  let { node, onclick, ondblclick }: Props = $props();

  let kindClass = $derived(`forge-graph-node--${node.kind}`);

  let flagClasses = $derived(() => {
    const cls: string[] = [];
    if (node.flags?.start) cls.push('forge-graph-node--start');
    if (node.flags?.unreachable) cls.push('forge-graph-node--unreachable');
    if (node.flags?.isolated) cls.push('forge-graph-node--isolated');
    if (node.flags?.orphaned) cls.push('forge-graph-node--orphaned');
    if (node.flags?.selected) cls.push('forge-graph-node--selected');
    return cls.join(' ');
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<g
  class="forge-graph-node {kindClass} {flagClasses()}"
  transform="translate({node.x - node.width / 2}, {node.y - node.height / 2})"
  onclick={() => onclick?.(node.id)}
  ondblclick={() => ondblclick?.(node.id)}
>
  <rect
    width={node.width}
    height={node.height}
    rx="4"
    ry="4"
    class="forge-graph-node__rect"
  />
  <text
    x={node.width / 2}
    y={node.height / 2}
    text-anchor="middle"
    dominant-baseline="central"
    class="forge-graph-node__label"
  >
    {node.label}
  </text>
  {#if node.flags?.start}
    <text
      x="6"
      y={node.height / 2}
      dominant-baseline="central"
      class="forge-graph-node__badge forge-graph-node__badge--start"
    >▶</text>
  {/if}
  {#if node.flags?.unreachable}
    <text
      x={node.width - 6}
      y={node.height / 2}
      text-anchor="end"
      dominant-baseline="central"
      class="forge-graph-node__badge forge-graph-node__badge--unreachable"
    >!</text>
  {/if}
</g>

<style>
  .forge-graph-node {
    cursor: pointer;
  }

  .forge-graph-node__rect {
    fill: var(--forge-graph-node-default);
    stroke: var(--forge-border-zone);
    stroke-width: 1.5;
    transition: fill 0.15s ease, stroke 0.15s ease;
  }

  .forge-graph-node:hover .forge-graph-node__rect {
    stroke: var(--forge-accent-primary);
    stroke-width: 2;
  }

  .forge-graph-node__label {
    fill: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    font-family: var(--forge-font-family-ui, 'Outfit', sans-serif);
    pointer-events: none;
    user-select: none;
  }

  .forge-graph-node__badge {
    font-size: 10px;
    pointer-events: none;
    user-select: none;
  }

  .forge-graph-node__badge--start {
    fill: var(--forge-graph-node-start, #e6a817);
  }

  .forge-graph-node__badge--unreachable {
    fill: var(--forge-graph-node-unreachable, #e94560);
    font-weight: bold;
  }

  /* Kind variants */
  .forge-graph-node--terminal .forge-graph-node__rect {
    fill: var(--forge-bg-tertiary);
    rx: 12;
    ry: 12;
  }

  /* Flag variants */
  .forge-graph-node--start .forge-graph-node__rect {
    stroke: var(--forge-graph-node-start, #e6a817);
    stroke-width: 2;
  }

  .forge-graph-node--unreachable .forge-graph-node__rect {
    stroke: var(--forge-graph-node-unreachable, #e94560);
    stroke-dasharray: 4 2;
    opacity: 0.7;
  }

  .forge-graph-node--isolated .forge-graph-node__rect {
    stroke-dasharray: 4 2;
    opacity: 0.5;
  }

  .forge-graph-node--orphaned .forge-graph-node__rect {
    stroke: var(--forge-semantic-warning, #e6a817);
    stroke-dasharray: 4 2;
  }

  .forge-graph-node--selected .forge-graph-node__rect {
    stroke: var(--forge-graph-node-selected);
    stroke-width: 2.5;
  }
</style>
