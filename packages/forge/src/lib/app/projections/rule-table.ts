/**
 * Rule table projection — flat rule rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface RuleRow {
  id: string;
  ruleId: string;
  conditionCount: number;
  effectCount: number;
  file: string;
  line: number;
}

export const ruleTableProjection: ProjectionDefinition<RuleRow[]> = {
  id: 'urd.projection.ruleTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): RuleRow[] => {
    const { factSet } = source;

    return factSet.rules.map((rule) => ({
      id: rule.rule_id,
      ruleId: rule.rule_id,
      conditionCount: rule.condition_reads.length,
      effectCount: rule.effect_writes.length,
      file: rule.span.file,
      line: rule.span.start_line,
    }));
  },
};
