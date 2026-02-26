/**
 * Workspace types.
 */

import type { ZoneTree, PersistedZoneState } from '../types';

export interface SerializedWorkspace {
  id: string;
  name: string;
  tree: ZoneTree;
  zoneStates: Record<string, PersistedZoneState>;
}

export interface SerializedWorkspaceSet {
  version: number;
  activeIndex: number;
  workspaces: SerializedWorkspace[];
}
