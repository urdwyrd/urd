/**
 * Application-level bus channel documentation for compiler events.
 *
 * The actual channel registrations are in ChannelManifest.ts (framework level)
 * since the PlaceholderBusMonitor needs to observe them. This file documents
 * the payload shapes for TypeScript consumers.
 */

export interface CompilerStartedPayload {
  compileId: string;
  timestamp: number;
  inputFileCount: number;
}

export interface CompilerCompletedPayload {
  compileId: string;
  durationMs: number;
  chunkHashes: Record<string, string>;
  worldCounts: {
    entities: number;
    locations: number;
    exits: number;
  };
}

export interface CompilerErrorPayload {
  compileId: string;
  error: string;
}

/**
 * Channel contract:
 * - compiler.started  → CompilerStartedPayload
 * - compiler.completed → CompilerCompletedPayload
 * - compiler.error    → CompilerErrorPayload
 */
