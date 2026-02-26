/**
 * File system abstraction — allows swapping Tauri FS for in-memory FS in tests.
 *
 * Framework-level interface. Implementations live in app/.
 */

export interface FileEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  isFile: boolean;
}

export interface FileStat {
  size: number;
  isDirectory: boolean;
  isFile: boolean;
  modifiedAt: number | null;
}

export interface FileChangeEvent {
  path: string;
  kind: 'create' | 'modify' | 'remove';
}

export type FileWatchCallback = (event: FileChangeEvent) => void;

export interface ForgeFileSystem {
  readFile(path: string): Promise<string>;
  writeFile(path: string, content: string): Promise<void>;
  listDirectory(path: string): Promise<FileEntry[]>;
  stat(path: string): Promise<FileStat>;
  exists(path: string): Promise<boolean>;
  mkdir(path: string): Promise<void>;

  /** Watch a file for changes. Returns an unwatch function. */
  watchFile(path: string, callback: FileWatchCallback): Promise<() => void>;

  /** Watch a directory for changes. Returns an unwatch function. */
  watchDirectory(path: string, callback: FileWatchCallback): Promise<() => void>;
}
