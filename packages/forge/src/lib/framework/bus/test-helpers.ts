/**
 * TestBus: records publishes, asserts payload size.
 * Used in unit tests to verify bus interactions without the real singleton.
 */

import { MessageBus } from './MessageBus';

export interface RecordedPublish {
  channelId: string;
  payload: unknown;
  timestamp: number;
}

export class TestBus extends MessageBus {
  readonly publishes: RecordedPublish[] = [];

  constructor() {
    super(true); // always dev mode in tests
  }

  override publish(channelId: string, payload: unknown): void {
    this.publishes.push({
      channelId,
      payload,
      timestamp: Date.now(),
    });
    super.publish(channelId, payload);
  }

  /** Returns all publishes for a specific channel. */
  publishesFor(channelId: string): RecordedPublish[] {
    return this.publishes.filter((p) => p.channelId === channelId);
  }

  /** Asserts that the given channel received exactly n publishes. */
  assertPublishCount(channelId: string, expected: number): void {
    const actual = this.publishesFor(channelId).length;
    if (actual !== expected) {
      throw new Error(
        `Expected ${expected} publishes on "${channelId}", got ${actual}`
      );
    }
  }

  /** Clears all recorded publishes. */
  reset(): void {
    this.publishes.length = 0;
  }
}
