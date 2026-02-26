/**
 * ActiveDialog — reactive state for which floating dialog is currently open.
 *
 * Uses Svelte 5 $state() so templates can react to changes.
 * Only one dialog is open at a time.
 */

export type DialogId = 'settings' | 'keybindings' | null;

class ActiveDialog {
  current: DialogId = $state(null);

  open(id: Exclude<DialogId, null>): void {
    this.current = id;
  }

  close(): void {
    this.current = null;
  }

  get isOpen(): boolean {
    return this.current !== null;
  }
}

export const activeDialog = new ActiveDialog();
