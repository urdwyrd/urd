<script lang="ts">
  /**
   * SettingsView — zone-mountable settings editor.
   *
   * Categorised sidebar. Immediate apply (no Save button).
   * Uses appSettings.set() which triggers settings.changed bus and debounced disk write.
   */

  import { appSettings, type AppSettings } from './AppSettingsService';
  import { setTheme, type ThemeName } from '../theme/ThemeEngine';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { }: Props = $props();

  type Category = 'Appearance' | 'Editor' | 'Behaviour';

  let activeCategory = $state<Category>('Appearance');
  let searchQuery = $state('');

  const categories: Category[] = ['Appearance', 'Editor', 'Behaviour'];

  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
    appSettings.set(key, value);
    // Side effects for specific settings
    if (key === 'theme') {
      setTheme(value as ThemeName);
    }
  }

  function resetAll(): void {
    appSettings.resetAll();
    setTheme(appSettings.get('theme'));
  }

  const filteredCategories = $derived.by(() => {
    if (!searchQuery.trim()) return categories;
    const q = searchQuery.toLowerCase();
    return categories.filter((cat) => {
      // Check if any setting label in this category matches
      return cat.toLowerCase().includes(q) || settingLabels(cat).some((l) => l.toLowerCase().includes(q));
    });
  });

  function settingLabels(cat: Category): string[] {
    switch (cat) {
      case 'Appearance': return ['Theme', 'UI Font Size'];
      case 'Editor': return ['Font Family', 'Font Size', 'Tab Size', 'Word Wrap', 'Line Numbers', 'Minimap'];
      case 'Behaviour': return ['Auto Save on Compile', 'Recompile Debounce', 'Open Last Project on Launch'];
    }
  }
</script>

<div class="forge-settings">
  <div class="forge-settings__sidebar">
    <input
      bind:value={searchQuery}
      class="forge-settings__search"
      type="text"
      placeholder="Search settings…"
      spellcheck="false"
    />
    {#each filteredCategories as cat}
      <button
        class="forge-settings__category"
        class:forge-settings__category--active={activeCategory === cat}
        onclick={() => { activeCategory = cat; }}
      >
        {cat}
      </button>
    {/each}
    <div class="forge-settings__sidebar-spacer"></div>
    <button class="forge-settings__reset" onclick={resetAll}>
      Reset All to Defaults
    </button>
  </div>

  <div class="forge-settings__content">
    {#if activeCategory === 'Appearance'}
      <h3 class="forge-settings__heading">Appearance</h3>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Theme</span>
        <select
          class="forge-settings__select"
          value={appSettings.get('theme')}
          onchange={(e) => updateSetting('theme', (e.target as HTMLSelectElement).value as ThemeName)}
        >
          <option value="gloaming">Gloaming (Dark)</option>
          <option value="parchment">Parchment (Light)</option>
        </select>
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">UI Font Size</span>
        <input
          class="forge-settings__input forge-settings__input--number"
          type="number"
          min="10"
          max="24"
          value={appSettings.get('uiFontSize')}
          onchange={(e) => updateSetting('uiFontSize', Number((e.target as HTMLInputElement).value))}
        />
      </label>
    {/if}

    {#if activeCategory === 'Editor'}
      <h3 class="forge-settings__heading">Editor</h3>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Font Family</span>
        <input
          class="forge-settings__input"
          type="text"
          value={appSettings.get('editorFontFamily')}
          onchange={(e) => updateSetting('editorFontFamily', (e.target as HTMLInputElement).value)}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Font Size</span>
        <input
          class="forge-settings__input forge-settings__input--number"
          type="number"
          min="8"
          max="32"
          value={appSettings.get('editorFontSize')}
          onchange={(e) => updateSetting('editorFontSize', Number((e.target as HTMLInputElement).value))}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Tab Size</span>
        <input
          class="forge-settings__input forge-settings__input--number"
          type="number"
          min="1"
          max="8"
          value={appSettings.get('editorTabSize')}
          onchange={(e) => updateSetting('editorTabSize', Number((e.target as HTMLInputElement).value))}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Word Wrap</span>
        <input
          class="forge-settings__checkbox"
          type="checkbox"
          checked={appSettings.get('editorWordWrap')}
          onchange={(e) => updateSetting('editorWordWrap', (e.target as HTMLInputElement).checked)}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Line Numbers</span>
        <input
          class="forge-settings__checkbox"
          type="checkbox"
          checked={appSettings.get('editorLineNumbers')}
          onchange={(e) => updateSetting('editorLineNumbers', (e.target as HTMLInputElement).checked)}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Minimap</span>
        <input
          class="forge-settings__checkbox"
          type="checkbox"
          checked={appSettings.get('editorMinimap')}
          onchange={(e) => updateSetting('editorMinimap', (e.target as HTMLInputElement).checked)}
        />
      </label>
    {/if}

    {#if activeCategory === 'Behaviour'}
      <h3 class="forge-settings__heading">Behaviour</h3>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Auto Save on Compile</span>
        <input
          class="forge-settings__checkbox"
          type="checkbox"
          checked={appSettings.get('autoSaveOnCompile')}
          onchange={(e) => updateSetting('autoSaveOnCompile', (e.target as HTMLInputElement).checked)}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Recompile Debounce (ms)</span>
        <input
          class="forge-settings__input forge-settings__input--number"
          type="number"
          min="100"
          max="2000"
          step="50"
          value={appSettings.get('recompileDebounceMs')}
          onchange={(e) => updateSetting('recompileDebounceMs', Number((e.target as HTMLInputElement).value))}
        />
      </label>

      <label class="forge-settings__row">
        <span class="forge-settings__label">Open Last Project on Launch</span>
        <input
          class="forge-settings__checkbox"
          type="checkbox"
          checked={appSettings.get('openLastProjectOnLaunch')}
          onchange={(e) => updateSetting('openLastProjectOnLaunch', (e.target as HTMLInputElement).checked)}
        />
      </label>
    {/if}
  </div>
</div>

<style>
  .forge-settings {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .forge-settings__sidebar {
    width: 180px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--forge-border-zone);
    padding: var(--forge-space-sm);
    gap: var(--forge-space-xs);
  }

  .forge-settings__search {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-xs);
    font-family: inherit;
    outline: none;
  }

  .forge-settings__search::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-settings__category {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-sm);
    font-family: inherit;
    cursor: pointer;
    text-align: left;
  }

  .forge-settings__category:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-settings__category--active {
    background: var(--forge-accent-primary);
    color: var(--forge-text-primary);
  }

  .forge-settings__sidebar-spacer {
    flex: 1;
  }

  .forge-settings__reset {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    font-family: inherit;
    cursor: pointer;
  }

  .forge-settings__reset:hover {
    color: var(--forge-status-error, #c66);
    border-color: var(--forge-status-error, #c66);
  }

  .forge-settings__content {
    flex: 1;
    overflow-y: auto;
    padding: var(--forge-space-lg);
  }

  .forge-settings__heading {
    margin: 0 0 var(--forge-space-lg);
    font-size: var(--forge-font-size-md);
    color: var(--forge-text-primary);
    font-weight: 600;
  }

  .forge-settings__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--forge-space-sm) 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .forge-settings__label {
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-sm);
  }

  .forge-settings__select {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    font-family: inherit;
    cursor: pointer;
  }

  .forge-settings__input {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    font-family: inherit;
    outline: none;
  }

  .forge-settings__input--number {
    width: 80px;
    text-align: right;
  }

  .forge-settings__checkbox {
    width: 16px;
    height: 16px;
    accent-color: var(--forge-accent-primary);
    cursor: pointer;
  }
</style>
