import { describe, it, expect, beforeEach } from 'vitest';
import { FocusService } from './FocusService';

describe('FocusService', () => {
  let service: FocusService;

  beforeEach(() => {
    service = new FocusService();
  });

  it('starts in normal mode with no active zone', () => {
    expect(service.mode).toBe('normal');
    expect(service.activeZoneId).toBeNull();
    expect(service.activeZoneType).toBeNull();
    expect(service.activeDividerId).toBeNull();
  });

  it('focusZone() sets active zone', () => {
    service.focusZone('zone_a', 'forge.placeholder.info');

    expect(service.activeZoneId).toBe('zone_a');
    expect(service.activeZoneType).toBe('forge.placeholder.info');
  });

  it('focusZone() clears divider focus', () => {
    service.focusDivider('split_1');
    expect(service.activeDividerId).toBe('split_1');

    service.focusZone('zone_a', 'forge.placeholder.info');
    expect(service.activeDividerId).toBeNull();
  });

  it('focusZone() is a no-op for same zone', () => {
    service.focusZone('zone_a', 'forge.placeholder.info');
    // Calling again with same zone should not error
    service.focusZone('zone_a', 'forge.placeholder.info');
    expect(service.activeZoneId).toBe('zone_a');
  });

  it('focusDivider() sets active divider', () => {
    service.focusDivider('split_1');
    expect(service.activeDividerId).toBe('split_1');
  });

  it('clearDividerFocus() clears divider', () => {
    service.focusDivider('split_1');
    service.clearDividerFocus();
    expect(service.activeDividerId).toBeNull();
  });

  it('pushMode/popMode manages focus mode stack', () => {
    expect(service.mode).toBe('normal');

    service.pushMode('commandPalette');
    expect(service.mode).toBe('commandPalette');

    service.pushMode('dialog');
    expect(service.mode).toBe('dialog');

    service.popMode();
    expect(service.mode).toBe('commandPalette');

    service.popMode();
    expect(service.mode).toBe('normal');
  });

  it('popMode() does not go below normal', () => {
    service.popMode(); // already at normal
    expect(service.mode).toBe('normal');
  });

  it('pushMode() rejects "normal"', () => {
    service.pushMode('normal');
    // Should still just be one normal on the stack
    expect(service.mode).toBe('normal');
  });

  it('clearFocus() resets everything', () => {
    service.focusZone('zone_a', 'info');
    service.focusDivider('split_1');
    service.pushMode('commandPalette');

    service.clearFocus();

    expect(service.activeZoneId).toBeNull();
    expect(service.activeZoneType).toBeNull();
    expect(service.activeDividerId).toBeNull();
    expect(service.mode).toBe('normal');
  });
});
