/**
 * propertyDependencyIndex passthrough projection — exposes the full
 * PropertyDependencyIndex through the projection registry for editor
 * extensions (hover tooltips, analysis panels, etc.).
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, PropertyDependencyIndex } from '$lib/app/compiler/types';

export const propertyDependencyIndexProjection: ProjectionDefinition<PropertyDependencyIndex> = {
  id: 'urd.projection.propertyDependencyIndex',
  depends: ['propertyDependencyIndex'],
  compute: (source: ResolvedCompilerOutput): PropertyDependencyIndex => {
    return source.propertyDependencyIndex;
  },
};
