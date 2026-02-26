<script lang="ts">
  /**
   * Welcome screen — shown when no project is open.
   * Replaces the workspace layout entirely.
   */

  import { projectManager } from './ProjectManager.svelte';

  let recentProjects = $derived(projectManager.getRecentProjects());

  async function handleOpenProject() {
    try {
      // Import Tauri dialog dynamically to avoid breaking non-Tauri environments
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        title: 'Open Urd Project',
      });
      if (selected) {
        await projectManager.openPath(selected as string);
      }
    } catch {
      // Browser dev mode fallback — open a mock project
      await projectManager.openPath('/mock/urd-project');
    }
  }

  async function handleOpenRecent(path: string) {
    await projectManager.openPath(path);
  }

  function handleRemoveRecent(e: MouseEvent, path: string) {
    e.stopPropagation();
    projectManager.removeFromRecent(path);
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString('en-GB', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  }
</script>

<div class="forge-welcome">
  <div class="forge-welcome__content">
    <h1 class="forge-welcome__title">Urd Forge</h1>
    <p class="forge-welcome__subtitle">Desktop IDE for Urd Schema Markdown</p>

    <div class="forge-welcome__actions">
      <button class="forge-welcome__btn forge-welcome__btn--primary" onclick={handleOpenProject}>
        Open Project
      </button>
      <button class="forge-welcome__btn" onclick={handleOpenProject}>
        New Project
      </button>
    </div>

    {#if recentProjects.length > 0}
      <div class="forge-welcome__recent">
        <h2 class="forge-welcome__recent-title">Recent Projects</h2>
        <ul class="forge-welcome__recent-list">
          {#each recentProjects as project}
            <li class="forge-welcome__recent-item">
              <button
                class="forge-welcome__recent-btn"
                onclick={() => handleOpenRecent(project.path)}
              >
                <span class="forge-welcome__recent-name">{project.name}</span>
                <span class="forge-welcome__recent-path forge-selectable">{project.path}</span>
                <span class="forge-welcome__recent-date">{formatDate(project.lastOpened)}</span>
              </button>
              <button
                class="forge-welcome__recent-remove"
                onclick={(e) => handleRemoveRecent(e, project.path)}
                title="Remove from recent"
              >
                ×
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
</div>

<style>
  .forge-welcome {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: var(--forge-bg-primary);
  }

  .forge-welcome__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--forge-space-xl);
    max-width: 480px;
    width: 100%;
    padding: var(--forge-space-2xl);
  }

  .forge-welcome__title {
    font-size: 28px;
    font-weight: var(--forge-font-weight-bold);
    color: var(--forge-text-primary);
    letter-spacing: -0.02em;
  }

  .forge-welcome__subtitle {
    font-size: var(--forge-font-size-md);
    color: var(--forge-text-secondary);
    margin-top: calc(-1 * var(--forge-space-md));
  }

  .forge-welcome__actions {
    display: flex;
    gap: var(--forge-space-md);
    margin-top: var(--forge-space-lg);
  }

  .forge-welcome__btn {
    padding: var(--forge-space-md) var(--forge-space-xl);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
    cursor: pointer;
    transition: background 0.15s;
  }

  .forge-welcome__btn:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-welcome__btn--primary {
    background: var(--forge-accent-primary);
    border-color: var(--forge-accent-primary);
    color: white;
  }

  .forge-welcome__btn--primary:hover {
    opacity: 0.9;
  }

  .forge-welcome__recent {
    width: 100%;
    margin-top: var(--forge-space-xl);
  }

  .forge-welcome__recent-title {
    font-size: var(--forge-font-size-sm);
    font-weight: var(--forge-font-weight-medium);
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--forge-space-md);
  }

  .forge-welcome__recent-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .forge-welcome__recent-item {
    display: flex;
    align-items: center;
    border-radius: var(--forge-radius-md);
    transition: background 0.1s;
  }

  .forge-welcome__recent-item:hover {
    background: var(--forge-bg-secondary);
  }

  .forge-welcome__recent-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-md);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    text-align: left;
    cursor: pointer;
  }

  .forge-welcome__recent-name {
    font-size: var(--forge-font-size-md);
    font-weight: var(--forge-font-weight-medium);
  }

  .forge-welcome__recent-path {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 380px;
  }

  .forge-welcome__recent-date {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-welcome__recent-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 16px;
    cursor: pointer;
    opacity: 0;
    margin-right: var(--forge-space-sm);
  }

  .forge-welcome__recent-item:hover .forge-welcome__recent-remove {
    opacity: 1;
  }

  .forge-welcome__recent-remove:hover {
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }
</style>
