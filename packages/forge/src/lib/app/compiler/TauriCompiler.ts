/**
 * Tauri compiler service — invokes the Rust compiler via IPC.
 *
 * Calls the `compile_project` Tauri command with the buffer map contents.
 * Only available when running inside Tauri (guarded).
 */

import type { CompilerService, CompilerOutput } from './types';

export class TauriCompiler implements CompilerService {
  async compile(buffers: Record<string, string>): Promise<CompilerOutput> {
    if (!('__TAURI_INTERNALS__' in window)) {
      throw new Error('TauriCompiler: not running inside Tauri');
    }

    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<CompilerOutput>('compile_project', { buffers });
    return result;
  }
}
