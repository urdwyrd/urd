/**
 * BSP Zone Tree — data structure, reducer, and invariant assertions.
 *
 * All layout mutations go through zoneTreeReducer. No code outside
 * the reducer may mutate the tree. This guarantees layout bugs are
 * caught immediately at the mutation site.
 */

import type { ZoneTree, SplitNode, LeafNode, ZoneTreeAction } from '../types';

let nextId = 1;

export function generateZoneId(): string {
  return `zone_${(nextId++).toString(36)}`;
}

export function generateSplitId(): string {
  return `split_${(nextId++).toString(36)}`;
}

/** Reset ID counter (for tests only). */
export function resetIdCounter(start = 1): void {
  nextId = start;
}

// ===== Tree queries =====

export function isLeaf(node: ZoneTree): node is LeafNode {
  return node.kind === 'leaf';
}

export function isSplit(node: ZoneTree): node is SplitNode {
  return node.kind === 'split';
}

/** Find a leaf by zone ID. */
export function findLeaf(tree: ZoneTree, zoneId: string): LeafNode | null {
  if (isLeaf(tree)) {
    return tree.id === zoneId ? tree : null;
  }
  return findLeaf(tree.children[0], zoneId) ?? findLeaf(tree.children[1], zoneId);
}

/** Find a split by divider ID. */
export function findSplit(tree: ZoneTree, splitId: string): SplitNode | null {
  if (isLeaf(tree)) return null;
  if (tree.id === splitId) return tree;
  return findSplit(tree.children[0], splitId) ?? findSplit(tree.children[1], splitId);
}

/** Find the parent split node that contains a given child ID. */
export function findParent(tree: ZoneTree, childId: string): SplitNode | null {
  if (isLeaf(tree)) return null;
  if (tree.children[0].id === childId || tree.children[1].id === childId) {
    return tree;
  }
  return findParent(tree.children[0], childId) ?? findParent(tree.children[1], childId);
}

/** Collect all leaf nodes in the tree. */
export function collectLeaves(tree: ZoneTree): LeafNode[] {
  if (isLeaf(tree)) return [tree];
  return [...collectLeaves(tree.children[0]), ...collectLeaves(tree.children[1])];
}

/** Collect all split nodes in the tree. */
export function collectSplits(tree: ZoneTree): SplitNode[] {
  if (isLeaf(tree)) return [];
  return [tree, ...collectSplits(tree.children[0]), ...collectSplits(tree.children[1])];
}

// ===== Tree invariant assertions =====

export function assertInvariants(tree: ZoneTree): void {
  const leafIds = new Set<string>();
  const splitIds = new Set<string>();

  function walk(node: ZoneTree): void {
    if (isLeaf(node)) {
      // Invariant 1: all leaf node IDs are unique
      if (leafIds.has(node.id)) {
        throw new Error(`ZoneTree invariant violation: duplicate leaf ID "${node.id}"`);
      }
      leafIds.add(node.id);
    } else {
      // Invariant 2: all split node IDs are unique
      if (splitIds.has(node.id)) {
        throw new Error(`ZoneTree invariant violation: duplicate split ID "${node.id}"`);
      }
      splitIds.add(node.id);

      // Invariant 3: a split node always has exactly two children, no identical refs
      if (node.children.length !== 2) {
        throw new Error(`ZoneTree invariant violation: split "${node.id}" has ${node.children.length} children`);
      }
      if (node.children[0] === node.children[1]) {
        throw new Error(`ZoneTree invariant violation: split "${node.id}" has identical child references`);
      }

      walk(node.children[0]);
      walk(node.children[1]);
    }
  }

  // Invariant 4: the root is valid
  if (!tree) {
    throw new Error('ZoneTree invariant violation: tree is null/undefined');
  }

  walk(tree);
}

// ===== Tree reducer =====

export function zoneTreeReducer(tree: ZoneTree, action: ZoneTreeAction): ZoneTree {
  const newTree = applyAction(tree, action);
  if (import.meta.env.DEV) {
    assertInvariants(newTree);
  }
  return newTree;
}

function applyAction(tree: ZoneTree, action: ZoneTreeAction): ZoneTree {
  switch (action.type) {
    case 'split':
      return applySplit(tree, action.zoneId, action.direction);
    case 'join':
      return applyJoin(tree, action.dividerId, action.keep);
    case 'swap':
      return applySwap(tree, action.dividerId);
    case 'resize':
      return applyResize(tree, action.dividerId, action.ratio);
    case 'changeType':
      return applyChangeType(tree, action.zoneId, action.newTypeId);
    case 'resetDivider':
      return applyResize(tree, action.dividerId, 0.5);
  }
}

function applySplit(tree: ZoneTree, zoneId: string, direction: 'horizontal' | 'vertical'): ZoneTree {
  return mapTree(tree, (node) => {
    if (isLeaf(node) && node.id === zoneId) {
      const newLeaf: LeafNode = {
        kind: 'leaf',
        id: generateZoneId(),
        zoneTypeId: node.zoneTypeId,
      };
      const split: SplitNode = {
        kind: 'split',
        id: generateSplitId(),
        direction,
        ratio: 0.5,
        children: [{ ...node }, newLeaf],
      };
      return split;
    }
    return node;
  });
}

function applyJoin(tree: ZoneTree, dividerId: string, keep: 'first' | 'second'): ZoneTree {
  return mapTree(tree, (node) => {
    if (isSplit(node) && node.id === dividerId) {
      const kept = keep === 'first' ? node.children[0] : node.children[1];
      return kept;
    }
    return node;
  });
}

function applySwap(tree: ZoneTree, dividerId: string): ZoneTree {
  return mapTree(tree, (node) => {
    if (isSplit(node) && node.id === dividerId) {
      // Swap semantics: exchange zoneTypeId and zone state between siblings.
      // Zone IDs do not move — they are persistence keys.
      const [first, second] = node.children;
      if (isLeaf(first) && isLeaf(second)) {
        return {
          ...node,
          children: [
            { ...first, zoneTypeId: second.zoneTypeId, singletonRef: second.singletonRef },
            { ...second, zoneTypeId: first.zoneTypeId, singletonRef: first.singletonRef },
          ] as [ZoneTree, ZoneTree],
        };
      }
      // For non-leaf children, swap the subtrees entirely
      return {
        ...node,
        children: [node.children[1], node.children[0]] as [ZoneTree, ZoneTree],
      };
    }
    return node;
  });
}

function applyResize(tree: ZoneTree, dividerId: string, ratio: number): ZoneTree {
  const clamped = Math.max(0.1, Math.min(0.9, ratio));
  return mapTree(tree, (node) => {
    if (isSplit(node) && node.id === dividerId) {
      return { ...node, ratio: clamped };
    }
    return node;
  });
}

function applyChangeType(tree: ZoneTree, zoneId: string, newTypeId: string): ZoneTree {
  return mapTree(tree, (node) => {
    if (isLeaf(node) && node.id === zoneId) {
      return { ...node, zoneTypeId: newTypeId };
    }
    return node;
  });
}

/** Deep-map a tree, calling `fn` on each node (post-order). */
function mapTree(tree: ZoneTree, fn: (node: ZoneTree) => ZoneTree): ZoneTree {
  if (isLeaf(tree)) {
    return fn(tree);
  }
  const mapped: SplitNode = {
    ...tree,
    children: [
      mapTree(tree.children[0], fn),
      mapTree(tree.children[1], fn),
    ] as [ZoneTree, ZoneTree],
  };
  return fn(mapped);
}

// ===== Factory helpers =====

export function createLeaf(zoneTypeId: string, id?: string): LeafNode {
  return {
    kind: 'leaf',
    id: id ?? generateZoneId(),
    zoneTypeId,
  };
}

export function createSplit(
  direction: 'horizontal' | 'vertical',
  first: ZoneTree,
  second: ZoneTree,
  ratio = 0.5,
  id?: string,
): SplitNode {
  return {
    kind: 'split',
    id: id ?? generateSplitId(),
    direction,
    ratio,
    children: [first, second],
  };
}
