/**
 * Buffer map — the source of truth for compilation input.
 *
 * Tracks file contents in memory with dirty state per file.
 * When a file is edited, the buffer is marked dirty. The recompile
 * pipeline reads from here, not from disk.
 */

type BufferChangeCallback = (path: string) => void;

interface BufferEntry {
  content: string;
  dirty: boolean;
}

export class BufferMap {
  private buffers = new Map<string, BufferEntry>();
  private listeners = new Set<BufferChangeCallback>();

  /** Get a file's content from the buffer. */
  get(path: string): string | undefined {
    return this.buffers.get(path)?.content;
  }

  /** Set a file's content. Marks it as dirty. */
  set(path: string, content: string): void {
    const existing = this.buffers.get(path);
    if (existing?.content === content) return;

    this.buffers.set(path, { content, dirty: true });
    this.notify(path);
  }

  /** Load a file's content without marking it dirty (initial load from disk). */
  load(path: string, content: string): void {
    this.buffers.set(path, { content, dirty: false });
  }

  /** Remove a file from the buffer. */
  remove(path: string): void {
    if (this.buffers.delete(path)) {
      this.notify(path);
    }
  }

  /** Get all file contents as a record. */
  getAll(): Record<string, string> {
    const result: Record<string, string> = {};
    for (const [path, entry] of this.buffers) {
      result[path] = entry.content;
    }
    return result;
  }

  /** Get all file paths. */
  paths(): string[] {
    return Array.from(this.buffers.keys());
  }

  /** Returns true if the file has unsaved changes. */
  isDirty(path: string): boolean {
    return this.buffers.get(path)?.dirty ?? false;
  }

  /** Returns true if any file in the buffer map is dirty. */
  hasAnyDirty(): boolean {
    for (const entry of this.buffers.values()) {
      if (entry.dirty) return true;
    }
    return false;
  }

  /** Mark a file as clean (after saving to disk). */
  markClean(path: string): void {
    const entry = this.buffers.get(path);
    if (entry) {
      entry.dirty = false;
    }
  }

  /** Mark all files as clean. */
  markAllClean(): void {
    for (const entry of this.buffers.values()) {
      entry.dirty = false;
    }
  }

  /** Clear all buffers (e.g., when closing a project). */
  clear(): void {
    this.buffers.clear();
  }

  /** Number of files in the buffer. */
  get size(): number {
    return this.buffers.size;
  }

  /** Subscribe to buffer changes. Returns unsubscribe function. */
  subscribe(callback: BufferChangeCallback): () => void {
    this.listeners.add(callback);
    return () => {
      this.listeners.delete(callback);
    };
  }

  private notify(path: string): void {
    for (const listener of this.listeners) {
      try {
        listener(path);
      } catch (err) {
        console.error('BufferMap listener error:', err);
      }
    }
  }
}

/** Singleton buffer map. */
export const bufferMap = new BufferMap();
