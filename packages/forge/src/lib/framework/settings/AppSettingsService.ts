/**
 * Application settings service.
 *
 * Loads from / saves to the OS config directory via Tauri path API.
 * Corruption-safe: if settings.json fails to parse, backs it up and loads defaults.
 * Publishes settings.changed on the bus when values change.
 */

import { bus } from '../bus/MessageBus';
import type { ThemeName } from '../theme/ThemeEngine';

// ===== Types =====

export interface RecentProject {
  name: string;
  path: string;
  lastOpened: number; // epoch ms
}

export interface AppSettings {
  theme: ThemeName;
  uiFontSize: number;
  editorFontFamily: string;
  editorFontSize: number;
  editorTabSize: number;
  editorWordWrap: boolean;
  editorLineNumbers: boolean;
  editorMinimap: boolean;
  autoSaveOnCompile: boolean;
  recompileDebounceMs: number;
  openLastProjectOnLaunch: boolean;
  recentProjects: RecentProject[];
  keybindingOverrides: Record<string, string>;
  lastProjectPath: string | null;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: 'gloaming',
  uiFontSize: 13,
  editorFontFamily: 'JetBrains Mono',
  editorFontSize: 13,
  editorTabSize: 2,
  editorWordWrap: false,
  editorLineNumbers: true,
  editorMinimap: false,
  autoSaveOnCompile: true,
  recompileDebounceMs: 300,
  openLastProjectOnLaunch: true,
  recentProjects: [],
  keybindingOverrides: {},
  lastProjectPath: null,
};

// ===== File I/O abstraction (swappable for tests) =====

export interface SettingsIO {
  read(): Promise<string | null>;
  write(content: string): Promise<void>;
  backup(content: string): Promise<void>;
}

/**
 * Tauri-based settings IO. Uses @tauri-apps/api for filesystem access.
 * Lazily imported to avoid breaking non-Tauri environments (tests, SSR).
 */
export class TauriSettingsIO implements SettingsIO {
  private configDir: string | null = null;

  private async getConfigDir(): Promise<string> {
    if (this.configDir) return this.configDir;
    const { appConfigDir } = await import('@tauri-apps/api/path');
    this.configDir = await appConfigDir();
    return this.configDir;
  }

  async read(): Promise<string | null> {
    try {
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const dir = await this.getConfigDir();
      return await readTextFile(`${dir}settings.json`);
    } catch {
      return null;
    }
  }

  async write(content: string): Promise<void> {
    const { writeTextFile, mkdir } = await import('@tauri-apps/plugin-fs');
    const dir = await this.getConfigDir();
    await mkdir(dir, { recursive: true });
    await writeTextFile(`${dir}settings.json`, content);
  }

  async backup(content: string): Promise<void> {
    const { writeTextFile, mkdir } = await import('@tauri-apps/plugin-fs');
    const dir = await this.getConfigDir();
    await mkdir(dir, { recursive: true });
    await writeTextFile(`${dir}settings.backup.json`, content);
  }
}

/** In-memory IO for tests. */
export class MemorySettingsIO implements SettingsIO {
  data: string | null = null;
  backupData: string | null = null;

  async read(): Promise<string | null> {
    return this.data;
  }

  async write(content: string): Promise<void> {
    this.data = content;
  }

  async backup(content: string): Promise<void> {
    this.backupData = content;
  }
}

// ===== Service =====

export class AppSettingsService {
  private settings: AppSettings = { ...DEFAULT_SETTINGS };
  private io: SettingsIO;
  private saveTimer: ReturnType<typeof setTimeout> | null = null;
  private loaded = false;

  constructor(io?: SettingsIO) {
    this.io = io ?? new TauriSettingsIO();
  }

  get current(): Readonly<AppSettings> {
    return this.settings;
  }

  get<K extends keyof AppSettings>(key: K): AppSettings[K] {
    return this.settings[key];
  }

  set<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
    const previousValue = this.settings[key];
    if (previousValue === value) return;

    this.settings = { ...this.settings, [key]: value };

    if (bus.hasChannel('settings.changed')) {
      bus.publish('settings.changed', { key, value, previousValue });
    }

    this.debouncedSave();
  }

  reset(key: keyof AppSettings): void {
    this.set(key, DEFAULT_SETTINGS[key] as AppSettings[typeof key]);
  }

  resetAll(): void {
    this.settings = { ...DEFAULT_SETTINGS };
    this.debouncedSave();
  }

  /** Load settings from disk. Call once at startup. */
  async load(): Promise<void> {
    if (this.loaded) return;

    const raw = await this.io.read();
    if (raw === null) {
      // No settings file — use defaults
      this.loaded = true;
      return;
    }

    try {
      const parsed = JSON.parse(raw);
      // Merge with defaults to pick up new keys from future versions
      this.settings = { ...DEFAULT_SETTINGS, ...parsed };
      this.loaded = true;
    } catch {
      // Corruption — back up and load defaults
      console.warn('Settings file corrupted — loading defaults. Backup saved.');
      await this.io.backup(raw);
      this.settings = { ...DEFAULT_SETTINGS };
      this.loaded = true;
    }
  }

  /** Save immediately (bypassing debounce). Useful at shutdown. */
  async saveNow(): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    await this.io.write(JSON.stringify(this.settings, null, 2));
  }

  private debouncedSave(): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
    }
    this.saveTimer = setTimeout(() => {
      this.saveNow().catch((err) => {
        console.error('Failed to save settings:', err);
      });
    }, 500);
  }
}

/** Singleton settings service. */
export const appSettings = new AppSettingsService();
