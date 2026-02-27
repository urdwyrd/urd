/**
 * File dependency graph projection — files as nodes, cross-file references as edges.
 *
 * Nodes: unique files from symbolTable.entries (kind='file').
 * Edges: cross-file references — when factSet exits/reads/writes span different files.
 * Depends on symbolTable, factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const fileDependencyGraphProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.fileDependencyGraph',
  depends: ['symbolTable', 'factSet'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { symbolTable, factSet } = source;

    // Collect unique files from symbol table
    const fileSet = new Set<string>();
    for (const entry of symbolTable.entries) {
      if (entry.file) {
        fileSet.add(entry.file);
      }
    }

    if (fileSet.size === 0) {
      return { nodes: [], edges: [] };
    }

    const nodes: ForgeGraphNode[] = Array.from(fileSet)
      .sort()
      .map((file) => ({
        id: file,
        label: file.split('/').pop() || file,
        kind: 'file' as const,
      }));

    // Build cross-file edges from factSet spans
    const edgeIds = new Set<string>();
    const edges: ForgeGraphEdge[] = [];

    function addEdge(fromFile: string, toFile: string, label: string): void {
      if (fromFile === toFile) return;
      if (!fileSet.has(fromFile) || !fileSet.has(toFile)) return;
      const edgeId = `${fromFile}→${toFile}`;
      if (edgeIds.has(edgeId)) return;
      edgeIds.add(edgeId);
      edges.push({
        id: edgeId,
        source: fromFile,
        target: toFile,
        label,
        kind: 'reference',
      });
    }

    // Cross-file exits: exit spans in one file referencing locations defined in another
    for (const exit of factSet.exits) {
      const fromFile = exit.span.file;
      // Find which file the target location is defined in
      const targetEntry = symbolTable.entries.find(
        (e) => e.id === exit.to_location || e.name === exit.to_location,
      );
      if (targetEntry && targetEntry.file) {
        addEdge(fromFile, targetEntry.file, 'exit');
      }
    }

    // Cross-file reads: property reads spanning different files from their definition
    for (const read of factSet.reads) {
      const readFile = read.span.file;
      const targetEntry = symbolTable.entries.find(
        (e) => e.id === read.entity_type || e.name === read.entity_type,
      );
      if (targetEntry && targetEntry.file) {
        addEdge(readFile, targetEntry.file, 'reads');
      }
    }

    // Cross-file writes: property writes spanning different files from their definition
    for (const write of factSet.writes) {
      const writeFile = write.span.file;
      const targetEntry = symbolTable.entries.find(
        (e) => e.id === write.entity_type || e.name === write.entity_type,
      );
      if (targetEntry && targetEntry.file) {
        addEdge(writeFile, targetEntry.file, 'writes');
      }
    }

    return { nodes, edges };
  },
};
