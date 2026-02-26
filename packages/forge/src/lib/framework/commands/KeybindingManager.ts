/**
 * Keyboard sovereignty — suppress browser shortcuts and route to commands.
 *
 * From the architecture doc §21.4:
 * - Intercepts browser shortcuts that would break IDE behaviour
 * - Routes keystrokes to the command registry
 * - Respects CodeMirror editor focus (only fires globalWhenEditorFocused commands)
 */

import { commandRegistry, formatKeybinding } from './CommandRegistry';

const SUPPRESSED_BROWSER_SHORTCUTS = [
  'ctrl+g',
  'ctrl+h',
  'ctrl+l',
  'ctrl+d',
  'ctrl+shift+i',
  'ctrl+u',
  'ctrl+p',
  'f5',
  'f7',
];

let installed = false;

export function installKeybindingManager(): () => void {
  if (installed) {
    console.warn('KeybindingManager already installed');
    return () => {};
  }

  const handler = (e: KeyboardEvent) => {
    const insideEditor = (e.target as HTMLElement)?.closest('.cm-editor') !== null;
    const key = formatKeybinding(e);

    if (SUPPRESSED_BROWSER_SHORTCUTS.includes(key)) {
      e.preventDefault();
    }

    if (insideEditor) {
      const command = commandRegistry.resolveByKeybinding(key);
      if (command?.globalWhenEditorFocused) {
        e.preventDefault();
        commandRegistry.execute(command.id);
      }
    } else {
      const command = commandRegistry.resolveByKeybinding(key);
      if (command) {
        e.preventDefault();
        commandRegistry.execute(command.id);
      }
    }
  };

  document.addEventListener('keydown', handler);
  installed = true;

  return () => {
    document.removeEventListener('keydown', handler);
    installed = false;
  };
}
