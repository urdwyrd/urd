/**
 * Workspace templates — predefined layouts for Writer, Engineer, World Builder, and Debug workflows.
 *
 * Writer: File Browser | Code Editor | Outline + Inspector
 * Engineer: File Browser | Code Editor | Property Spreadsheet + Diagnostic Spreadsheet
 * World Builder: Code Editor + Location Graph (top) | Entity Spreadsheet (bottom)
 * Debug: Play Panel + Event Log (left) | Code Editor + State Inspector (right)
 */

import type { ZoneTree } from '$lib/framework/types';
import { createLeaf, createSplit } from '$lib/framework/layout/ZoneTree';

/**
 * Writer workspace: File Browser (20%) | Code Editor (55%) | Outline + Inspector (25%)
 */
export function createWriterTemplate(): ZoneTree {
  return createSplit(
    'horizontal',
    createLeaf('urd.fileBrowser'),
    createSplit(
      'horizontal',
      createLeaf('urd.codeEditor'),
      createSplit(
        'vertical',
        createLeaf('urd.outlinePanel'),
        createLeaf('urd.propertyInspector'),
        0.5,
      ),
      0.7,
    ),
    0.2,
  );
}

/**
 * Engineer workspace: File Browser (20%) | Code Editor (50%) | Properties + Diagnostics (30%)
 */
export function createEngineerTemplate(): ZoneTree {
  return createSplit(
    'horizontal',
    createLeaf('urd.fileBrowser'),
    createSplit(
      'horizontal',
      createLeaf('urd.codeEditor'),
      createSplit(
        'vertical',
        createLeaf('urd.propertySpreadsheet'),
        createLeaf('urd.diagnosticSpreadsheet'),
        0.5,
      ),
      0.6,
    ),
    0.2,
  );
}

/**
 * World Builder workspace: Code Editor (50%) + Location Graph (50%) on top (65%)
 *                          Entity Spreadsheet on bottom (35%)
 */
export function createWorldBuilderTemplate(): ZoneTree {
  return createSplit(
    'vertical',
    createSplit(
      'horizontal',
      createLeaf('urd.codeEditor'),
      createLeaf('urd.locationGraph'),
      0.5,
    ),
    createLeaf('urd.entitySpreadsheet'),
    0.65,
  );
}

/**
 * Debug workspace: Play Panel (60%) + Event Log (40%) on left (50%)
 *                  Code Editor (60%) + State Inspector (40%) on right (50%)
 */
export function createDebugTemplate(): ZoneTree {
  return createSplit(
    'horizontal',
    createSplit(
      'vertical',
      createLeaf('urd.playPanel'),
      createLeaf('urd.eventLog'),
      0.6,
    ),
    createSplit(
      'vertical',
      createLeaf('urd.codeEditor'),
      createLeaf('urd.stateInspector'),
      0.6,
    ),
    0.5,
  );
}
