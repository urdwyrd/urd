/**
 * Mock compiler service — returns fixture data for development and testing.
 *
 * Simulates chunked output with content hashes. On repeated calls with
 * the same input, returns "unchanged" chunk hashes so the cache works.
 */

import type {
  CompilerService,
  CompilerOutput,
  Chunk,
  OutputHeader,
} from './types';
import fixtureData from '../../../../fixtures/basic-world.json';

let compileCounter = 0;

/** Simple hash for content — not cryptographic, just for change detection. */
function simpleHash(input: string): string {
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    const char = input.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash |= 0; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}

export class MockCompilerService implements CompilerService {
  private lastInputHash: string | null = null;
  private lastOutput: CompilerOutput | null = null;

  /** Simulated compile delay in milliseconds. Set to 0 for tests. */
  delayMs = 50;

  async compile(buffers: Record<string, string>, _entryFile?: string): Promise<CompilerOutput> {
    if (this.delayMs > 0) {
      await new Promise((resolve) => setTimeout(resolve, this.delayMs));
    }

    const inputHash = simpleHash(JSON.stringify(buffers));
    const now = Date.now();
    compileCounter++;

    // If input hasn't changed, return same output with same hashes
    if (this.lastInputHash === inputHash && this.lastOutput) {
      return {
        ...this.lastOutput,
        header: {
          ...this.lastOutput.header,
          compileId: `mock-${compileCounter}`,
          timestamp: now,
        },
      };
    }

    // Build fresh output from fixture, generating new hashes based on input
    const fixture = fixtureData as CompilerOutput;
    const chunks: Chunk[] = fixture.chunks.map((chunk) => ({
      ...chunk,
      contentHash: simpleHash(`${chunk.name}-${inputHash}`),
    }));

    const fileCount = Object.keys(buffers).length;
    const header: OutputHeader = {
      compileId: `mock-${compileCounter}`,
      timestamp: now,
      durationMs: this.delayMs,
      phaseTimings: fixture.header.phaseTimings,
      worldCounts: fixture.header.worldCounts,
      inputFileCount: fileCount,
    };

    const output: CompilerOutput = { header, chunks };
    this.lastInputHash = inputHash;
    this.lastOutput = output;

    return output;
  }
}
