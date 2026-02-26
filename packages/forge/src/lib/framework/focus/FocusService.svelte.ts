/**
 * Focus service — tracks the active zone and focus mode stack.
 *
 * The focus service is the authority on which zone is active and what
 * modal mode the application is in (normal, commandPalette, dialog, etc.).
 * Modal modes are a stack so nested modals work correctly.
 *
 * Uses Svelte 5 $state() runes so that getters are reactive in templates.
 */

import { bus } from '../bus/MessageBus';
import type { FocusMode } from '../types';

export class FocusService {
  private _activeZoneId: string | null = $state(null);
  private _activeZoneType: string | null = $state(null);
  private _activeDividerId: string | null = $state(null);
  private _modeStack: FocusMode[] = $state(['normal']);

  get activeZoneId(): string | null {
    return this._activeZoneId;
  }

  get activeZoneType(): string | null {
    return this._activeZoneType;
  }

  get activeDividerId(): string | null {
    return this._activeDividerId;
  }

  get mode(): FocusMode {
    return this._modeStack[this._modeStack.length - 1];
  }

  /** Focus a zone. Clears any active divider focus. */
  focusZone(zoneId: string, zoneType: string): void {
    const previousZoneId = this._activeZoneId;
    if (previousZoneId === zoneId) return;

    this._activeZoneId = zoneId;
    this._activeZoneType = zoneType;
    this._activeDividerId = null;

    if (bus.hasChannel('focus.zoneChanged')) {
      bus.publish('focus.zoneChanged', {
        zoneId,
        zoneType,
        previousZoneId,
      });
    }
  }

  /** Focus a divider (for keyboard-driven resize). */
  focusDivider(dividerId: string): void {
    this._activeDividerId = dividerId;
  }

  /** Clear divider focus. */
  clearDividerFocus(): void {
    this._activeDividerId = null;
  }

  /** Push a modal focus mode onto the stack. */
  pushMode(mode: FocusMode): void {
    if (mode === 'normal') {
      console.warn('FocusService: cannot push "normal" mode — it is the base mode');
      return;
    }
    this._modeStack = [...this._modeStack, mode];
  }

  /** Pop the current modal focus mode. Returns to the previous mode. */
  popMode(): FocusMode {
    if (this._modeStack.length <= 1) {
      return 'normal';
    }
    const popped = this._modeStack[this._modeStack.length - 1];
    this._modeStack = this._modeStack.slice(0, -1);
    return popped;
  }

  /** Clear all focus state and reset mode to normal. */
  clearFocus(): void {
    this._activeZoneId = null;
    this._activeZoneType = null;
    this._activeDividerId = null;
    this._modeStack = ['normal'];
  }
}

/** Singleton focus service. */
export const focusService = new FocusService();
