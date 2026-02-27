/**
 * Rule trigger network projection — rules and properties as nodes, condition/effect links as edges.
 *
 * Nodes: rules (kind='section') + properties (kind='property') from factSet.
 * Edges: condition_reads link rules→properties (kind='reference'),
 *        effect_writes link rules→properties (kind='normal').
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const ruleTriggerNetworkProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.ruleTriggerNetwork',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { factSet } = source;

    if (!factSet.rules.length) {
      return { nodes: [], edges: [] };
    }

    const nodeIds = new Set<string>();
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];

    function ensureNode(id: string, label: string, kind: 'section' | 'property'): void {
      if (!nodeIds.has(id)) {
        nodeIds.add(id);
        nodes.push({ id, label, kind });
      }
    }

    for (const rule of factSet.rules) {
      const ruleId = `rule:${rule.rule_id}`;
      ensureNode(ruleId, rule.rule_id, 'section');

      // Condition reads: rule reads from properties
      for (const readIdx of rule.condition_reads) {
        const read = factSet.reads[readIdx];
        if (!read) continue;
        const propId = `prop:${read.entity_type}.${read.property}`;
        ensureNode(propId, `${read.entity_type}.${read.property}`, 'property');
        edges.push({
          id: `cond_${rule.rule_id}_r${readIdx}`,
          source: propId,
          target: ruleId,
          label: `${read.operator} ${read.value_literal}`,
          kind: 'reference',
        });
      }

      // Effect writes: rule writes to properties
      for (const writeIdx of rule.effect_writes) {
        const write = factSet.writes[writeIdx];
        if (!write) continue;
        const propId = `prop:${write.entity_type}.${write.property}`;
        ensureNode(propId, `${write.entity_type}.${write.property}`, 'property');
        edges.push({
          id: `eff_${rule.rule_id}_w${writeIdx}`,
          source: ruleId,
          target: propId,
          label: `${write.operator} ${write.value_expr}`,
          kind: 'normal',
        });
      }
    }

    return { nodes, edges };
  },
};
