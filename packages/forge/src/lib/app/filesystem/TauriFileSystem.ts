/**
 * Tauri-backed file system implementation.
 *
 * Uses @tauri-apps/plugin-fs for all file operations.
 * File watching is stubbed for Phase 2 — will be implemented
 * when the Rust file watcher integration is ready.
 */

import type {
  ForgeFileSystem,
  FileEntry,
  FileStat,
  FileWatchCallback,
} from '$lib/framework/filesystem/FileSystem';

export class TauriFileSystem implements ForgeFileSystem {
  async readFile(path: string): Promise<string> {
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    return readTextFile(path);
  }

  async writeFile(path: string, content: string): Promise<void> {
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    await writeTextFile(path, content);
  }

  async listDirectory(path: string): Promise<FileEntry[]> {
    const { readDir } = await import('@tauri-apps/plugin-fs');
    const entries = await readDir(path);
    return entries.map((entry) => ({
      name: entry.name,
      path: `${path}/${entry.name}`,
      isDirectory: entry.isDirectory,
      isFile: entry.isFile,
    }));
  }

  async stat(path: string): Promise<FileStat> {
    const { stat } = await import('@tauri-apps/plugin-fs');
    const info = await stat(path);
    return {
      size: info.size,
      isDirectory: info.isDirectory,
      isFile: info.isFile,
      modifiedAt: info.mtime ? new Date(info.mtime).getTime() : null,
    };
  }

  async exists(path: string): Promise<boolean> {
    const { exists } = await import('@tauri-apps/plugin-fs');
    return exists(path);
  }

  async mkdir(path: string): Promise<void> {
    const { mkdir } = await import('@tauri-apps/plugin-fs');
    await mkdir(path, { recursive: true });
  }

  async watchFile(_path: string, _callback: FileWatchCallback): Promise<() => void> {
    // Stubbed for Phase 2 — will use Tauri file watcher
    console.warn('TauriFileSystem.watchFile: not yet implemented');
    return () => {};
  }

  async watchDirectory(_path: string, _callback: FileWatchCallback): Promise<() => void> {
    // Stubbed for Phase 2 — will use Tauri file watcher
    console.warn('TauriFileSystem.watchDirectory: not yet implemented');
    return () => {};
  }
}
