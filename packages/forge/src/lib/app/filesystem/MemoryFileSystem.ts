/**
 * In-memory file system implementation for tests and browser dev mode.
 */

import type {
  ForgeFileSystem,
  FileEntry,
  FileStat,
  FileChangeEvent,
  FileWatchCallback,
} from '$lib/framework/filesystem/FileSystem';

interface MemoryNode {
  content: string | null; // null for directories
  isDirectory: boolean;
  modifiedAt: number;
}

export class MemoryFileSystem implements ForgeFileSystem {
  private files = new Map<string, MemoryNode>();
  private watchers = new Map<string, Set<FileWatchCallback>>();

  /** Pre-populate the filesystem with files. Paths should use forward slashes. */
  seed(entries: Record<string, string>): void {
    for (const [path, content] of Object.entries(entries)) {
      this.files.set(normalisePath(path), {
        content,
        isDirectory: false,
        modifiedAt: Date.now(),
      });
      // Ensure parent directories exist
      this.ensureParents(path);
    }
  }

  /** Pre-populate a directory. */
  seedDirectory(path: string): void {
    this.files.set(normalisePath(path), {
      content: null,
      isDirectory: true,
      modifiedAt: Date.now(),
    });
  }

  async readFile(path: string): Promise<string> {
    const node = this.files.get(normalisePath(path));
    if (!node || node.isDirectory) {
      throw new Error(`File not found: ${path}`);
    }
    return node.content!;
  }

  async writeFile(path: string, content: string): Promise<void> {
    const normalised = normalisePath(path);
    const existed = this.files.has(normalised);
    this.files.set(normalised, {
      content,
      isDirectory: false,
      modifiedAt: Date.now(),
    });
    this.ensureParents(path);
    this.notifyWatchers(normalised, existed ? 'modify' : 'create');
  }

  async listDirectory(path: string): Promise<FileEntry[]> {
    const normalised = normalisePath(path);
    const prefix = normalised.endsWith('/') ? normalised : normalised + '/';
    const entries: FileEntry[] = [];
    const seen = new Set<string>();

    for (const [filePath, node] of this.files) {
      if (filePath.startsWith(prefix) && filePath !== normalised) {
        // Only include direct children (no nested paths)
        const rest = filePath.slice(prefix.length);
        const firstSlash = rest.indexOf('/');
        const childName = firstSlash === -1 ? rest : rest.slice(0, firstSlash);
        if (!seen.has(childName)) {
          seen.add(childName);
          const childPath = prefix + childName;
          const childNode = this.files.get(childPath);
          entries.push({
            name: childName,
            path: childPath,
            isDirectory: (childNode?.isDirectory) ?? (firstSlash !== -1),
            isFile: (childNode != null ? !childNode.isDirectory : firstSlash === -1),
          });
        }
      }
    }

    return entries;
  }

  async stat(path: string): Promise<FileStat> {
    const node = this.files.get(normalisePath(path));
    if (!node) {
      throw new Error(`Path not found: ${path}`);
    }
    return {
      size: node.content?.length ?? 0,
      isDirectory: node.isDirectory,
      isFile: !node.isDirectory,
      modifiedAt: node.modifiedAt,
    };
  }

  async exists(path: string): Promise<boolean> {
    return this.files.has(normalisePath(path));
  }

  async mkdir(path: string): Promise<void> {
    const normalised = normalisePath(path);
    if (!this.files.has(normalised)) {
      this.files.set(normalised, {
        content: null,
        isDirectory: true,
        modifiedAt: Date.now(),
      });
    }
    this.ensureParents(path);
  }

  async watchFile(path: string, callback: FileWatchCallback): Promise<() => void> {
    const normalised = normalisePath(path);
    return this.addWatcher(normalised, callback);
  }

  async watchDirectory(path: string, callback: FileWatchCallback): Promise<() => void> {
    const normalised = normalisePath(path);
    return this.addWatcher(normalised, callback);
  }

  private addWatcher(path: string, callback: FileWatchCallback): () => void {
    if (!this.watchers.has(path)) {
      this.watchers.set(path, new Set());
    }
    this.watchers.get(path)!.add(callback);
    return () => {
      this.watchers.get(path)?.delete(callback);
    };
  }

  private notifyWatchers(path: string, kind: FileChangeEvent['kind']): void {
    // Notify file watchers
    const fileWatchers = this.watchers.get(path);
    if (fileWatchers) {
      for (const cb of fileWatchers) {
        cb({ path, kind });
      }
    }

    // Notify parent directory watchers
    const parentDir = path.slice(0, path.lastIndexOf('/'));
    if (parentDir) {
      const dirWatchers = this.watchers.get(parentDir);
      if (dirWatchers) {
        for (const cb of dirWatchers) {
          cb({ path, kind });
        }
      }
    }
  }

  private ensureParents(path: string): void {
    const parts = normalisePath(path).split('/');
    for (let i = 1; i < parts.length; i++) {
      const parentPath = parts.slice(0, i).join('/');
      if (parentPath && !this.files.has(parentPath)) {
        this.files.set(parentPath, {
          content: null,
          isDirectory: true,
          modifiedAt: Date.now(),
        });
      }
    }
  }
}

function normalisePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '');
}
