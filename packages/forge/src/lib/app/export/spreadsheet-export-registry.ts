/**
 * Spreadsheet export registry — maps spreadsheet view IDs to projection IDs
 * and column definitions for centralised CSV and print export.
 *
 * This avoids modifying each spreadsheet view — the export commands look up
 * the focused zone type here, fetch the projection data, and call the export utility.
 */

import type { ColumnDefinition } from '$lib/app/views/spreadsheets/_shared/types';

export interface SpreadsheetExportEntry {
  /** The projection ID to fetch data from. */
  projectionId: string;
  /** Column definitions for CSV header + cell extraction. */
  columns: ColumnDefinition<Record<string, unknown>>[];
  /** Human-readable name for filename generation. */
  label: string;
}

/**
 * Registry mapping spreadsheet view IDs to their export metadata.
 * Each entry describes how to extract and format data for CSV/print export.
 */
export const spreadsheetExportRegistry: Record<string, SpreadsheetExportEntry> = {
  'urd.entitySpreadsheet': {
    projectionId: 'entityTable',
    label: 'Entities',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'propertyCount', label: 'Properties' },
      { key: 'dependencyCount', label: 'Dependencies' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.typeSpreadsheet': {
    projectionId: 'typeTable',
    label: 'Types',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'kind', label: 'Kind' },
      { key: 'entityCount', label: 'Entities' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.propertySpreadsheet': {
    projectionId: 'propertyTable',
    label: 'Properties',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'owner', label: 'Owner' },
      { key: 'type', label: 'Type' },
      { key: 'defaultValue', label: 'Default' },
      { key: 'file', label: 'File' },
    ],
  },
  'urd.locationSpreadsheet': {
    projectionId: 'locationTable',
    label: 'Locations',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'exitCount', label: 'Exits' },
      { key: 'entityCount', label: 'Entities' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.sectionSpreadsheet': {
    projectionId: 'sectionTable',
    label: 'Sections',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'kind', label: 'Kind' },
      { key: 'childCount', label: 'Children' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.diagnosticSpreadsheet': {
    projectionId: 'diagnosticsByFile',
    label: 'Diagnostics',
    columns: [
      { key: 'severity', label: 'Severity' },
      { key: 'message', label: 'Message' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
      { key: 'code', label: 'Code' },
    ],
  },
  'urd.choiceSpreadsheet': {
    projectionId: 'choiceTable',
    label: 'Choices',
    columns: [
      { key: 'label', label: 'Label' },
      { key: 'sourceSection', label: 'Source' },
      { key: 'targetSection', label: 'Target' },
      { key: 'condition', label: 'Condition' },
      { key: 'file', label: 'File' },
    ],
  },
  'urd.ruleSpreadsheet': {
    projectionId: 'ruleTable',
    label: 'Rules',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'trigger', label: 'Trigger' },
      { key: 'conditionCount', label: 'Conditions' },
      { key: 'actionCount', label: 'Actions' },
      { key: 'file', label: 'File' },
    ],
  },
  'urd.exitSpreadsheet': {
    projectionId: 'exitTable',
    label: 'Exits',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'from', label: 'From' },
      { key: 'to', label: 'To' },
      { key: 'bidirectional', label: 'Bidirectional' },
      { key: 'file', label: 'File' },
    ],
  },
  'urd.jumpSpreadsheet': {
    projectionId: 'jumpTable',
    label: 'Jumps',
    columns: [
      { key: 'label', label: 'Label' },
      { key: 'source', label: 'Source' },
      { key: 'target', label: 'Target' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.readSpreadsheet': {
    projectionId: 'readTable',
    label: 'Reads',
    columns: [
      { key: 'property', label: 'Property' },
      { key: 'reader', label: 'Reader' },
      { key: 'context', label: 'Context' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.writeSpreadsheet': {
    projectionId: 'writeTable',
    label: 'Writes',
    columns: [
      { key: 'property', label: 'Property' },
      { key: 'writer', label: 'Writer' },
      { key: 'value', label: 'Value' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.sequenceSpreadsheet': {
    projectionId: 'sequenceTable',
    label: 'Sequences',
    columns: [
      { key: 'name', label: 'Name' },
      { key: 'stepCount', label: 'Steps' },
      { key: 'owner', label: 'Owner' },
      { key: 'file', label: 'File' },
      { key: 'line', label: 'Line' },
    ],
  },
  'urd.fileSpreadsheet': {
    projectionId: 'fileTable',
    label: 'Files',
    columns: [
      { key: 'path', label: 'Path' },
      { key: 'sectionCount', label: 'Sections' },
      { key: 'entityCount', label: 'Entities' },
      { key: 'diagnosticCount', label: 'Diagnostics' },
      { key: 'lineCount', label: 'Lines' },
    ],
  },
};
