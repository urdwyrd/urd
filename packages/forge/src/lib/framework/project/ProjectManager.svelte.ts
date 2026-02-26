/**
 * Project manager — open, close, recent projects.
 */

import { bus } from '../bus/MessageBus';
import { appSettings, type RecentProject } from '../settings/AppSettingsService';

export class ProjectManager {
  currentPath: string | null = $state(null);
  currentName: string | null = $state(null);
  isOpen: boolean = $derived(this.currentPath !== null);

  async openPath(path: string): Promise<void> {
    const name = path.split(/[/\\]/).pop() ?? path;

    this.currentPath = path;
    this.currentName = name;

    // Update recent projects
    const recents = appSettings.get('recentProjects').filter((r) => r.path !== path);
    recents.unshift({ name, path, lastOpened: Date.now() });
    // Keep at most 10
    appSettings.set('recentProjects', recents.slice(0, 10));
    appSettings.set('lastProjectPath', path);

    // Set window title (Tauri only)
    await this.setWindowTitle(`${name} — Urd Forge`);

    if (bus.hasChannel('project.opened')) {
      bus.publish('project.opened', { path, name });
    }
  }

  close(): void {
    const path = this.currentPath;
    this.currentPath = null;
    this.currentName = null;

    // Reset window title
    this.setWindowTitle('Urd Forge');

    if (bus.hasChannel('project.closed')) {
      bus.publish('project.closed', { path });
    }
  }

  private async setWindowTitle(title: string): Promise<void> {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        await getCurrentWindow().setTitle(title);
      } catch {
        // Silent fallback in browser dev mode
      }
    }
  }

  getRecentProjects(): RecentProject[] {
    return appSettings.get('recentProjects');
  }

  removeFromRecent(path: string): void {
    const recents = appSettings.get('recentProjects').filter((r) => r.path !== path);
    appSettings.set('recentProjects', recents);
  }

  showWelcome(): void {
    this.close();
  }
}

/** Singleton project manager. */
export const projectManager = new ProjectManager();
