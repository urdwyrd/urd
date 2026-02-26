/**
 * Theme engine — manages Gloaming (dark) and Parchment (light) themes.
 * Sets data-theme attribute on the root element. All components use token references.
 */

import { bus } from '../bus/MessageBus';

export type ThemeName = 'gloaming' | 'parchment';

const THEME_ATTRIBUTE = 'data-theme';
const DEFAULT_THEME: ThemeName = 'gloaming';

let currentTheme: ThemeName = DEFAULT_THEME;

export function getCurrentTheme(): ThemeName {
  return currentTheme;
}

export function setTheme(theme: ThemeName): void {
  const previous = currentTheme;
  currentTheme = theme;
  document.documentElement.setAttribute(THEME_ATTRIBUTE, theme);

  // Update colour-scheme for native UI elements
  document.documentElement.style.colorScheme = theme === 'gloaming' ? 'dark' : 'light';

  if (bus.hasChannel('theme.changed')) {
    bus.publish('theme.changed', { theme, previous });
  }
}

export function toggleTheme(): void {
  setTheme(currentTheme === 'gloaming' ? 'parchment' : 'gloaming');
}

/** Initialise theme from a saved preference. Call once at startup. */
export function initTheme(savedTheme?: ThemeName): void {
  setTheme(savedTheme ?? DEFAULT_THEME);
}
