/**
 * Workspace layout persistence — saves/loads workspace layouts to disk.
 *
 * Per-project: .forge/workspaces.json (auto-saved on layout/state changes)
 * Global default: default-layout.json in OS config dir (user-triggered)
 *
 * Debounced 1000ms saves. Corruption-safe with try/catch fallbacks.
 */

import type { ForgeFileSystem } from '$lib/framework/filesystem/FileSystem';
import type { WorkspaceManager } from '$lib/framework/workspace/WorkspaceManager.svelte';
import type { SerializedWorkspaceSet } from '$lib/framework/workspace/types';

// ===== Global layout IO abstraction =====

export interface GlobalLayoutIO {
  read(): Promise<string | null>;
  write(content: string): Promise<void>;
}

/**
 * Tauri-based global layout IO. Resolves default-layout.json in appConfigDir.
 */
export class TauriGlobalLayoutIO implements GlobalLayoutIO {
  private configDir: string | null = null;

  private async getConfigDir(): Promise<string> {
    if (this.configDir) return this.configDir;
    const { appConfigDir } = await import('@tauri-apps/api/path');
    this.configDir = await appConfigDir();
    return this.configDir;
  }

  private async resolve(): Promise<string> {
    const { join } = await import('@tauri-apps/api/path');
    const dir = await this.getConfigDir();
    return await join(dir, 'default-layout.json');
  }

  async read(): Promise<string | null> {
    try {
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const path = await this.resolve();
      return await readTextFile(path);
    } catch {
      return null;
    }
  }

  async write(content: string): Promise<void> {
    const { writeTextFile, mkdir } = await import('@tauri-apps/plugin-fs');
    const dir = await this.getConfigDir();
    await mkdir(dir, { recursive: true });
    const path = await this.resolve();
    await writeTextFile(path, content);
  }
}

/** In-memory global layout IO for browser dev mode and tests. */
export class MemoryGlobalLayoutIO implements GlobalLayoutIO {
  data: string | null = null;

  async read(): Promise<string | null> {
    return this.data;
  }

  async write(content: string): Promise<void> {
    this.data = content;
  }
}

// ===== Service =====

export class WorkspacePersistenceService {
  private fs: ForgeFileSystem;
  private workspaceManager: WorkspaceManager;
  private globalIO: GlobalLayoutIO;
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(fs: ForgeFileSystem, workspaceManager: WorkspaceManager, globalIO: GlobalLayoutIO) {
    this.fs = fs;
    this.workspaceManager = workspaceManager;
    this.globalIO = globalIO;
  }

  // ===== Per-project persistence =====

  /**
   * Load workspace layout from .forge/workspaces.json.
   * Returns true if a valid layout was loaded, false otherwise.
   */
  async loadProject(projectPath: string): Promise<boolean> {
    const filePath = `${projectPath}/.forge/workspaces.json`;
    try {
      const exists = await this.fs.exists(filePath);
      if (!exists) return false;

      const raw = await this.fs.readFile(filePath);
      const parsed = JSON.parse(raw) as SerializedWorkspaceSet;
      return this.workspaceManager.deserialize(parsed);
    } catch (err) {
      console.warn('WorkspacePersistence: failed to load project layout:', err);
      return false;
    }
  }

  /** Save workspace layout to .forge/workspaces.json (debounced 1000ms). */
  saveProject(projectPath: string): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
    }
    this.saveTimer = setTimeout(() => {
      this.saveProjectNow(projectPath).catch((err) => {
        console.error('WorkspacePersistence: failed to save project layout:', err);
      });
    }, 1000);
  }

  /** Save immediately (bypass debounce). Call on project close / app shutdown. */
  async saveProjectNow(projectPath: string): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    const filePath = `${projectPath}/.forge/workspaces.json`;
    try {
      const data = this.workspaceManager.serialize();
      const content = JSON.stringify(data, null, 2);
      await this.fs.writeFile(filePath, content);
    } catch (err) {
      console.error('WorkspacePersistence: failed to write project layout:', err);
    }
  }

  // ===== Global default layout =====

  /**
   * Load the global default layout from OS config dir.
   * Returns true if a valid layout was loaded, false otherwise.
   */
  async loadGlobalDefault(): Promise<boolean> {
    try {
      const raw = await this.globalIO.read();
      if (raw === null) return false;

      const parsed = JSON.parse(raw) as SerializedWorkspaceSet;
      return this.workspaceManager.deserialize(parsed);
    } catch (err) {
      console.warn('WorkspacePersistence: failed to load global default layout:', err);
      return false;
    }
  }

  /** Save the current layout as the global default. */
  async saveGlobalDefault(): Promise<void> {
    try {
      const data = this.workspaceManager.serialize();
      const content = JSON.stringify(data, null, 2);
      await this.globalIO.write(content);
    } catch (err) {
      console.error('WorkspacePersistence: failed to save global default layout:', err);
    }
  }

  /** Cancel any pending save timer. */
  dispose(): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
  }
}
