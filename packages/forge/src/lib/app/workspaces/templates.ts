/**
 * Workspace templates — predefined layouts for Writer and Engineer workflows.
 *
 * Writer: File Browser | Code Editor | Outline + Inspector
 * Engineer: File Browser | Code Editor | Property Spreadsheet + Diagnostic Spreadsheet
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
