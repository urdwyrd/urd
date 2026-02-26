import { describe, it, expect, beforeEach } from 'vitest';
import { NavigationBroker } from './NavigationBroker';
import { viewRegistry } from '../views/ViewRegistry';
import { focusService } from '../focus/FocusService.svelte';

describe('NavigationBroker', () => {
  let broker: NavigationBroker;

  beforeEach(() => {
    broker = new NavigationBroker();
  });

  it('queues intents for unregistered views', () => {
    const result = broker.navigate({
      targetViewId: 'urd.codeEditor',
    });

    expect(result.resolved).toBe(false);
    expect(broker.queueSize).toBe(1);
  });

  it('navigates to existing multi view in focused zone', () => {
    // Register a multi view
    viewRegistry.register({
      id: 'test.multi',
      name: 'Test Multi',
      icon: 'T',
      category: 'Test',
      component: () => Promise.resolve({ default: {} as never }),
      navigationStrategy: 'multi',
      stateVersion: 1,
      defaultState: null,
    });

    // Set a focused zone
    focusService.focusZone('zone_test', 'forge.placeholder.colour');

    const result = broker.navigate({ targetViewId: 'test.multi' });
    expect(result.resolved).toBe(true);
    if (result.resolved) {
      expect(result.zoneId).toBe('zone_test');
    }
  });

  it('returns unresolved when no zone is available for multi', () => {
    viewRegistry.register({
      id: 'test.multi2',
      name: 'Test Multi 2',
      icon: 'T',
      category: 'Test',
      component: () => Promise.resolve({ default: {} as never }),
      navigationStrategy: 'multi',
      stateVersion: 1,
      defaultState: null,
    });

    focusService.clearFocus();

    const result = broker.navigate({ targetViewId: 'test.multi2' });
    expect(result.resolved).toBe(false);
  });

  it('drainQueue returns still-pending intents', () => {
    broker.navigate({ targetViewId: 'missing.view' });
    broker.navigate({ targetViewId: 'another.missing' });

    const pending = broker.drainQueue();
    // Both should still be pending since views aren't registered
    expect(pending.length).toBe(2);
  });
});
