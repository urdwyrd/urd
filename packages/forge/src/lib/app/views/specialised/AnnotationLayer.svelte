<script lang="ts">
  /**
   * AnnotationLayer — user annotations list.
   *
   * Stores annotations in local $state (not persisted to the file system yet).
   * Has an "Add Note" button with text input. Shows list of annotations
   * with file:line and note text. Click navigates to source.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { AnnotationService, type PersistedAnnotation } from '$lib/app/services/AnnotationService';
  import { fileSystem } from '$lib/app/bootstrap';
  import { projectManager } from '$lib/framework/project/ProjectManager.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface Annotation {
    id: string;
    file: string;
    line: number;
    note: string;
    createdAt: number;
  }

  let annotations: Annotation[] = $state([]);
  let showForm = $state(false);
  let noteText = $state('');
  let noteFile = $state('');
  let noteLine = $state(1);

  const annotationService = new AnnotationService(fileSystem);

  // Load annotations when project opens
  let unsubOpen: (() => void) | undefined;
  let unsubClose: (() => void) | undefined;

  onMount(() => {
    // If a project is already open, load annotations immediately
    if (projectManager.isOpen) {
      loadAnnotations();
    }

    unsubOpen = bus.subscribe('project.opened', () => {
      loadAnnotations();
    });

    unsubClose = bus.subscribe('project.closed', () => {
      annotations = [];
    });
  });

  onDestroy(() => {
    unsubOpen?.();
    unsubClose?.();
    annotationService.dispose();
  });

  async function loadAnnotations(): Promise<void> {
    const path = projectManager.currentPath;
    if (!path) return;
    const loaded = await annotationService.load(path);
    annotations = loaded;
  }

  function persistAnnotations(): void {
    const path = projectManager.currentPath;
    if (!path) return;
    annotationService.save(path, annotations as PersistedAnnotation[]);
  }

  function handleAddNote(): void {
    if (!noteText.trim()) return;

    const annotation: Annotation = {
      id: crypto.randomUUID(),
      file: noteFile.trim() || '(no file)',
      line: noteLine,
      note: noteText.trim(),
      createdAt: Date.now(),
    };

    annotations = [...annotations, annotation];
    persistAnnotations();
    noteText = '';
    noteFile = '';
    noteLine = 1;
    showForm = false;
  }

  function handleRemove(id: string): void {
    annotations = annotations.filter((a) => a.id !== id);
    persistAnnotations();
  }

  function handleNavigate(annotation: Annotation): void {
    if (annotation.file === '(no file)') return;
    navigationBroker.navigate({
      targetViewId: 'urd.codeEditor',
      params: { path: annotation.file, line: annotation.line },
    });
  }

  function handleCancel(): void {
    showForm = false;
    noteText = '';
    noteFile = '';
    noteLine = 1;
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }
</script>

<div class="forge-annotations">
  <div class="forge-annotations__toolbar">
    <span class="forge-annotations__title">Annotations</span>
    <div class="forge-annotations__spacer"></div>
    <span class="forge-annotations__count">{annotations.length}</span>
    <button
      class="forge-annotations__btn forge-annotations__btn--add"
      onclick={() => { showForm = !showForm; }}
      title="Add a new annotation"
    >
      + Add Note
    </button>
  </div>

  {#if showForm}
    <div class="forge-annotations__form">
      <div class="forge-annotations__form-row">
        <input
          class="forge-annotations__input forge-annotations__input--file"
          type="text"
          placeholder="File path (optional)"
          bind:value={noteFile}
        />
        <input
          class="forge-annotations__input forge-annotations__input--line"
          type="number"
          min="1"
          placeholder="Line"
          bind:value={noteLine}
        />
      </div>
      <textarea
        class="forge-annotations__textarea"
        placeholder="Enter your note..."
        bind:value={noteText}
        rows="3"
      ></textarea>
      <div class="forge-annotations__form-actions">
        <button class="forge-annotations__btn" onclick={handleCancel}>Cancel</button>
        <button
          class="forge-annotations__btn forge-annotations__btn--save"
          onclick={handleAddNote}
          disabled={!noteText.trim()}
        >
          Save
        </button>
      </div>
    </div>
  {/if}

  {#if annotations.length === 0 && !showForm}
    <div class="forge-annotations__empty">
      <p>No annotations yet</p>
      <p class="forge-annotations__hint">
        Click "Add Note" to create annotations for specific file locations
      </p>
    </div>
  {:else}
    <div class="forge-annotations__list">
      {#each annotations as annotation (annotation.id)}
        <div class="forge-annotations__item">
          <div class="forge-annotations__item-header">
            <button
              class="forge-annotations__item-location"
              onclick={() => handleNavigate(annotation)}
              title="Navigate to source"
            >
              {annotation.file}:{annotation.line}
            </button>
            <span class="forge-annotations__item-time">{formatTime(annotation.createdAt)}</span>
            <button
              class="forge-annotations__btn forge-annotations__btn--remove"
              onclick={() => handleRemove(annotation.id)}
              title="Remove annotation"
            >
              x
            </button>
          </div>
          <div class="forge-annotations__item-note">{annotation.note}</div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-annotations {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-annotations__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-annotations__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-annotations__spacer {
    flex: 1;
  }

  .forge-annotations__count {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-annotations__btn {
    padding: 2px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-annotations__btn:hover {
    background: var(--forge-bg-hover);
  }

  .forge-annotations__btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .forge-annotations__btn--add {
    color: var(--forge-accent-primary, #5b9bd5);
    border-color: var(--forge-accent-primary, #5b9bd5);
  }

  .forge-annotations__btn--save {
    color: var(--forge-runtime-play-active, #4caf50);
    border-color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-annotations__btn--remove {
    padding: 0 4px;
    border: none;
    background: none;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
  }

  .forge-annotations__btn--remove:hover {
    color: var(--forge-graph-node-unreachable, #e94560);
    background: none;
  }

  .forge-annotations__form {
    padding: var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
    background: var(--forge-bg-secondary);
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-sm);
    flex-shrink: 0;
  }

  .forge-annotations__form-row {
    display: flex;
    gap: var(--forge-space-sm);
  }

  .forge-annotations__input {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
  }

  .forge-annotations__input--file {
    flex: 1;
  }

  .forge-annotations__input--line {
    width: 60px;
  }

  .forge-annotations__textarea {
    padding: var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    resize: vertical;
    min-height: 48px;
  }

  .forge-annotations__textarea::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-annotations__form-actions {
    display: flex;
    gap: var(--forge-space-sm);
    justify-content: flex-end;
  }

  .forge-annotations__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-annotations__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-annotations__list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .forge-annotations__item {
    padding: var(--forge-space-sm) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
  }

  .forge-annotations__item:hover {
    background: var(--forge-bg-hover);
  }

  .forge-annotations__item-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-annotations__item-location {
    background: none;
    border: none;
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-accent-primary, #5b9bd5);
    cursor: pointer;
    padding: 0;
    text-align: left;
  }

  .forge-annotations__item-location:hover {
    text-decoration: underline;
  }

  .forge-annotations__item-time {
    margin-left: auto;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-annotations__item-note {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
    line-height: 1.4;
    white-space: pre-wrap;
  }
</style>
