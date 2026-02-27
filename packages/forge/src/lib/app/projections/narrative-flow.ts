/**
 * Narrative flow projection — section sequence with branching information.
 *
 * Builds a step-by-step view of the narrative, identifying terminal
 * sections (no outgoing jumps or choices), branching points, and
 * the edges between sections.
 *
 * Depends on factSet, symbolTable.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface NarrativeStep {
  sectionId: string;
  sectionName: string;
  choiceCount: number;
  jumpCount: number;
  isTerminal: boolean;
}

export interface NarrativeFlowData {
  steps: NarrativeStep[];
  branches: Array<{ from: string; to: string; label: string }>;
}

export const narrativeFlowProjection: ProjectionDefinition<NarrativeFlowData> = {
  id: 'urd.projection.narrativeFlow',
  depends: ['factSet', 'symbolTable'],
  compute: (source: ResolvedCompilerOutput): NarrativeFlowData => {
    const { factSet, symbolTable } = source;

    // Build a name lookup from symbol table
    const nameById = new Map<string, string>();
    for (const sym of symbolTable.entries) {
      if (sym.kind === 'section') {
        nameById.set(sym.id, sym.name);
      }
    }

    // Collect all section IDs from jumps, choices, and symbol table
    const sectionIds = new Set<string>();
    for (const sym of symbolTable.entries) {
      if (sym.kind === 'section') {
        sectionIds.add(sym.id);
      }
    }
    for (const jump of factSet.jumps) {
      sectionIds.add(jump.from_section);
      if (jump.target.id) sectionIds.add(jump.target.id);
    }
    for (const choice of factSet.choices) {
      sectionIds.add(choice.section);
    }

    // Count choices per section
    const choiceCounts = new Map<string, number>();
    for (const choice of factSet.choices) {
      choiceCounts.set(
        choice.section,
        (choiceCounts.get(choice.section) ?? 0) + 1,
      );
    }

    // Count jumps per section and collect branch edges
    const jumpCounts = new Map<string, number>();
    const branches: Array<{ from: string; to: string; label: string }> = [];

    for (const jump of factSet.jumps) {
      jumpCounts.set(
        jump.from_section,
        (jumpCounts.get(jump.from_section) ?? 0) + 1,
      );

      if (jump.target.id) {
        branches.push({
          from: jump.from_section,
          to: jump.target.id,
          label: jump.target.kind === 'named' ? 'jump' : jump.target.kind,
        });
      }
    }

    // Also add branches from choices that have jump indices
    for (const choice of factSet.choices) {
      for (const jumpIdx of choice.jump_indices) {
        const jump = factSet.jumps[jumpIdx];
        if (jump?.target.id) {
          branches.push({
            from: choice.section,
            to: jump.target.id,
            label: choice.label || 'choice',
          });
        }
      }
    }

    // Build steps
    const steps: NarrativeStep[] = [];
    for (const sectionId of sectionIds) {
      const choiceCount = choiceCounts.get(sectionId) ?? 0;
      const jumpCount = jumpCounts.get(sectionId) ?? 0;
      const isTerminal = choiceCount === 0 && jumpCount === 0;

      steps.push({
        sectionId,
        sectionName: nameById.get(sectionId) ?? sectionId,
        choiceCount,
        jumpCount,
        isTerminal,
      });
    }

    // Sort: non-terminal first, then alphabetical
    steps.sort((a, b) => {
      if (a.isTerminal !== b.isTerminal) return a.isTerminal ? 1 : -1;
      return a.sectionName.localeCompare(b.sectionName);
    });

    // Deduplicate branches
    const seen = new Set<string>();
    const uniqueBranches = branches.filter((b) => {
      const key = `${b.from}->${b.to}:${b.label}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });

    return {
      steps,
      branches: uniqueBranches,
    };
  },
};
