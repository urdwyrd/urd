import { describe, it, expect, beforeEach } from 'vitest';
import { SelectionContext, type SelectionItem } from './SelectionContext';

describe('SelectionContext', () => {
  let ctx: SelectionContext;

  beforeEach(() => {
    ctx = new SelectionContext();
  });

  it('starts with empty selection', () => {
    expect(ctx.state.items).toEqual([]);
    expect(ctx.state.sourceZoneId).toBeNull();
  });

  it('select() updates state', () => {
    const items: SelectionItem[] = [
      { kind: 'entity', id: 'ent_1', label: 'Player' },
    ];
    ctx.select(items, 'zone_a');

    expect(ctx.state.items).toEqual(items);
    expect(ctx.state.sourceZoneId).toBe('zone_a');
  });

  it('clear() resets to empty', () => {
    ctx.select([{ kind: 'entity', id: 'ent_1' }], 'zone_a');
    ctx.clear();

    expect(ctx.state.items).toEqual([]);
    expect(ctx.state.sourceZoneId).toBeNull();
  });

  it('clear() is a no-op when already empty', () => {
    const listener = { called: 0 };
    ctx.subscribe(() => { listener.called++; });
    // Initial replay counts as one call
    expect(listener.called).toBe(1);

    ctx.clear();
    // Should not notify again since already empty
    expect(listener.called).toBe(1);
  });

  it('subscribe() replays current state to new subscriber', () => {
    const items: SelectionItem[] = [{ kind: 'location', id: 'loc_1' }];
    ctx.select(items, 'zone_b');

    let received: SelectionItem[] = [];
    ctx.subscribe((state) => { received = state.items; });

    expect(received).toEqual(items);
  });

  it('subscribe() notifies on changes', () => {
    const history: number[] = [];
    ctx.subscribe((state) => { history.push(state.items.length); });

    ctx.select([{ kind: 'entity', id: 'e1' }]);
    ctx.select([{ kind: 'entity', id: 'e1' }, { kind: 'entity', id: 'e2' }]);
    ctx.clear();

    // [0 (replay), 1, 2, 0]
    expect(history).toEqual([0, 1, 2, 0]);
  });

  it('unsubscribe stops notifications', () => {
    const history: number[] = [];
    const unsub = ctx.subscribe((state) => { history.push(state.items.length); });

    ctx.select([{ kind: 'entity', id: 'e1' }]);
    unsub();
    ctx.select([{ kind: 'entity', id: 'e2' }]);

    expect(history).toEqual([0, 1]); // replay + one change, not the second
  });
});
