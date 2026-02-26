/**
 * Generic selection context — tracks the primary selection across zones.
 *
 * Views call `select()` to set the primary selection, which is published
 * on the bus as `selection.primary`. Other views subscribe to react to
 * selection changes (e.g., inspector panels).
 */

import { bus } from '../bus/MessageBus';

export interface SelectionItem {
  kind: string;
  id: string;
  label?: string;
  data?: Record<string, unknown>;
}

export interface SelectionState {
  items: SelectionItem[];
  sourceZoneId: string | null;
}

type SelectionListener = (state: SelectionState) => void;

const EMPTY_SELECTION: SelectionState = { items: [], sourceZoneId: null };

export class SelectionContext {
  private current: SelectionState = EMPTY_SELECTION;
  private listeners = new Set<SelectionListener>();

  /** Returns the current selection state. */
  get state(): Readonly<SelectionState> {
    return this.current;
  }

  /** Set the primary selection. Publishes `selection.primary` on the bus. */
  select(items: SelectionItem[], sourceZoneId: string | null = null): void {
    this.current = { items, sourceZoneId };
    this.notify();
    if (bus.hasChannel('selection.primary')) {
      bus.publish('selection.primary', {
        count: items.length,
        kinds: [...new Set(items.map((i) => i.kind))],
        sourceZoneId,
      });
    }
  }

  /** Clear the selection. */
  clear(): void {
    if (this.current.items.length === 0) return;
    this.current = EMPTY_SELECTION;
    this.notify();
    if (bus.hasChannel('selection.primary')) {
      bus.publish('selection.primary', { count: 0, kinds: [], sourceZoneId: null });
    }
  }

  /** Subscribe to selection changes. Returns unsubscribe function. */
  subscribe(listener: SelectionListener): () => void {
    this.listeners.add(listener);
    // Replay current state to new subscriber
    listener(this.current);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    for (const listener of this.listeners) {
      try {
        listener(this.current);
      } catch (err) {
        console.error('SelectionContext listener error:', err);
      }
    }
  }
}

/** Singleton selection context. */
export const selectionContext = new SelectionContext();
