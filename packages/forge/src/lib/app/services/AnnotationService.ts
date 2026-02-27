/**
 * Annotation persistence service — loads/saves annotations to .forge/annotations.json.
 *
 * Debounced 500ms saves. Corruption-safe with try/catch.
 */

import type { ForgeFileSystem } from '$lib/framework/filesystem/FileSystem';

export interface PersistedAnnotation {
  id: string;
  file: string;
  line: number;
  note: string;
  createdAt: number;
}

export class AnnotationService {
  private fs: ForgeFileSystem;
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(fs: ForgeFileSystem) {
    this.fs = fs;
  }

  /** Load annotations from .forge/annotations.json. Returns [] on missing/corrupt file. */
  async load(projectPath: string): Promise<PersistedAnnotation[]> {
    const filePath = `${projectPath}/.forge/annotations.json`;
    try {
      const exists = await this.fs.exists(filePath);
      if (!exists) return [];

      const raw = await this.fs.readFile(filePath);
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed as PersistedAnnotation[];
    } catch {
      console.warn('AnnotationService: failed to load annotations — returning empty list');
      return [];
    }
  }

  /** Save annotations to .forge/annotations.json (debounced 500ms). */
  save(projectPath: string, annotations: PersistedAnnotation[]): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
    }
    this.saveTimer = setTimeout(() => {
      this.saveNow(projectPath, annotations).catch((err) => {
        console.error('AnnotationService: failed to save annotations:', err);
      });
    }, 500);
  }

  /** Save immediately (bypass debounce). */
  async saveNow(projectPath: string, annotations: PersistedAnnotation[]): Promise<void> {
    const filePath = `${projectPath}/.forge/annotations.json`;
    try {
      const content = JSON.stringify(annotations, null, 2);
      await this.fs.writeFile(filePath, content);
    } catch (err) {
      console.error('AnnotationService: failed to write annotations:', err);
    }
  }

  /** Cancel any pending save. */
  dispose(): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
  }
}
