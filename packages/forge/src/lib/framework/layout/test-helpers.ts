/**
 * Layout test helpers — createTestTree(), assertTreeInvariants().
 */

import type { ZoneTree } from '../types';
import { createLeaf, createSplit, assertInvariants, resetIdCounter } from './ZoneTree';

/**
 * Create a simple test tree with the given number of zones.
 * All zones get a 'forge.placeholder.colour' type.
 */
export function createTestTree(zoneCount: number): ZoneTree {
  resetIdCounter();

  if (zoneCount < 1) {
    throw new Error('createTestTree requires at least 1 zone');
  }
  if (zoneCount === 1) {
    return createLeaf('forge.placeholder.colour');
  }

  // Build a balanced binary tree of splits
  let leaves: ZoneTree[] = [];
  for (let i = 0; i < zoneCount; i++) {
    leaves.push(createLeaf('forge.placeholder.colour'));
  }

  while (leaves.length > 1) {
    const merged: ZoneTree[] = [];
    for (let i = 0; i < leaves.length; i += 2) {
      if (i + 1 < leaves.length) {
        merged.push(createSplit(i % 4 < 2 ? 'horizontal' : 'vertical', leaves[i], leaves[i + 1]));
      } else {
        merged.push(leaves[i]);
      }
    }
    leaves = merged;
  }

  return leaves[0];
}

/** Re-export for convenience. */
export { assertInvariants as assertTreeInvariants } from './ZoneTree';
