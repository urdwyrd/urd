/**
 * Filter context — application-layer filter state.
 *
 * Views that support filtering subscribe to `filter.changed` on the bus
 * and apply the active filter to their display. The filter context holds
 * a union of possible filter types.
 */

import { bus } from '$lib/framework/bus/MessageBus';

export interface TextFilter {
  kind: 'text';
  query: string;
}

export interface TagFilter {
  kind: 'tag';
  tags: string[];
}

export interface SeverityFilter {
  kind: 'severity';
  levels: ('error' | 'warning' | 'info')[];
}

export type FilterItem = TextFilter | TagFilter | SeverityFilter;

export class FilterContext {
  private _active: FilterItem | null = null;

  get active(): FilterItem | null {
    return this._active;
  }

  set(filter: FilterItem): void {
    this._active = filter;
    if (bus.hasChannel('filter.changed')) {
      bus.publish('filter.changed', { filter });
    }
  }

  clear(): void {
    if (this._active === null) return;
    this._active = null;
    if (bus.hasChannel('filter.changed')) {
      bus.publish('filter.changed', { filter: null });
    }
  }
}

/** Singleton filter context. */
export const filterContext = new FilterContext();
