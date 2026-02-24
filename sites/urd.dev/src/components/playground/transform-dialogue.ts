/**
 * Transform FactSet jump and choice data into a dialogue flow graph.
 *
 * Nodes: unique section IDs from JumpEdge and ChoiceFact tuples,
 *        plus synthetic terminal nodes for 'end' and exit targets.
 * Edges: one per JumpEdge, labelled with owning choice (via jump_indices).
 * Flags: cross-reference URD432 diagnostics for impossible choice conditions.
 */

import type { FactSet, Diagnostic } from './compiler-bridge';
import type { GraphData, GraphNode, GraphEdge } from './graph-types';

/** Build a reverse map: jump index → owning ChoiceFact. */
function buildJumpToChoiceMap(facts: FactSet): Map<number, typeof facts.choices[number]> {
  const map = new Map<number, typeof facts.choices[number]>();
  for (const choice of facts.choices) {
    for (const jumpIdx of choice.jump_indices) {
      map.set(jumpIdx, choice);
    }
  }
  return map;
}

export function buildDialogueGraph(
  facts: FactSet,
  diagnostics: Diagnostic[],
): GraphData {
  const nodeMap = new Map<string, GraphNode>();
  const edges: GraphEdge[] = [];
  let hasEnd = false;

  function ensureSection(sectionId: string): void {
    if (!nodeMap.has(sectionId)) {
      nodeMap.set(sectionId, {
        id: sectionId,
        label: sectionId.split('/').pop() ?? sectionId,
        kind: 'section',
        flag: null,
      });
    }
  }

  // Parse URD432 diagnostics — extract section names with impossible choices.
  const orphanedSections = new Set<string>();
  for (const diag of diagnostics) {
    if (diag.code === 'URD432') {
      const match = diag.message.match(/Choice in section '([^']+)'/);
      if (match) orphanedSections.add(match[1]);
    }
  }

  // Collect section nodes from jumps.
  for (const jump of facts.jumps) {
    ensureSection(jump.from_section);
    if (jump.target.kind === 'section' && jump.target.id) {
      ensureSection(jump.target.id);
    }
    if (jump.target.kind === 'end') {
      hasEnd = true;
    }
  }

  // Collect section nodes from choices.
  for (const choice of facts.choices) {
    ensureSection(choice.section);
  }

  // Add synthetic terminal nodes.
  if (hasEnd) {
    nodeMap.set('__end__', {
      id: '__end__',
      label: 'end',
      kind: 'end',
      flag: null,
    });
  }

  // Add exit terminal nodes.
  const exitTargets = new Set<string>();
  for (const jump of facts.jumps) {
    if (jump.target.kind === 'exit' && jump.target.id) {
      exitTargets.add(jump.target.id);
    }
  }
  for (const exitId of exitTargets) {
    const nodeId = `__exit_${exitId}`;
    nodeMap.set(nodeId, {
      id: nodeId,
      label: `\u2192 ${exitId}`,
      kind: 'end',
      flag: null,
    });
  }

  // Build reverse map from jump index to owning choice.
  const jumpToChoice = buildJumpToChoiceMap(facts);

  // Build edges from all JumpEdges.
  for (let i = 0; i < facts.jumps.length; i++) {
    const jump = facts.jumps[i];

    let to: string;
    if (jump.target.kind === 'section' && jump.target.id) {
      to = jump.target.id;
    } else if (jump.target.kind === 'exit' && jump.target.id) {
      to = `__exit_${jump.target.id}`;
    } else if (jump.target.kind === 'end') {
      to = '__end__';
    } else {
      continue;
    }

    const ownerChoice = jumpToChoice.get(i) ?? null;

    edges.push({
      from: jump.from_section,
      to,
      label: ownerChoice?.label ?? '',
      conditional: ownerChoice
        ? ownerChoice.condition_reads.length > 0
        : false,
    });
  }

  // Apply orphaned flags.
  for (const [id, node] of nodeMap) {
    if (node.kind !== 'section') continue;
    // URD432 message contains the section name (not the full compiled ID).
    const sectionName = id.split('/').pop() ?? id;
    if (orphanedSections.has(sectionName)) {
      node.flag = 'orphaned';
    }
  }

  return { nodes: Array.from(nodeMap.values()), edges };
}
