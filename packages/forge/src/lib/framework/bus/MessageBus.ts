/**
 * Channel-registered pub/sub message bus.
 *
 * Rules from the architecture doc:
 *  - Signals only, never data. 4KB dev-mode payload assertion.
 *  - Channels must be registered before use.
 *  - retainLast channels replay the last published value to new subscribers.
 */

const DEV_PAYLOAD_LIMIT = 4096;

export interface ChannelDefinition {
  id: string;
  domain: string;
  retainLast: boolean;
}

export type BusCallback = (payload: unknown) => void;

interface Subscription {
  callback: BusCallback;
  id: number;
}

interface ChannelState {
  definition: ChannelDefinition;
  subscribers: Subscription[];
  lastPayload: unknown;
  hasPublished: boolean;
}

export class MessageBus {
  private channels = new Map<string, ChannelState>();
  private nextSubId = 1;
  private isDev: boolean;

  constructor(isDev = true) {
    this.isDev = isDev;
  }

  registerChannel(definition: ChannelDefinition): void {
    if (this.channels.has(definition.id)) {
      throw new Error(`Bus channel already registered: ${definition.id}`);
    }
    this.channels.set(definition.id, {
      definition,
      subscribers: [],
      lastPayload: undefined,
      hasPublished: false,
    });
  }

  hasChannel(channelId: string): boolean {
    return this.channels.has(channelId);
  }

  publish(channelId: string, payload: unknown): void {
    const channel = this.channels.get(channelId);
    if (!channel) {
      throw new Error(`Bus channel not registered: ${channelId}. Register before publishing.`);
    }

    if (this.isDev) {
      this.assertPayloadSize(channelId, payload);
    }

    channel.lastPayload = payload;
    channel.hasPublished = true;

    for (const sub of channel.subscribers) {
      try {
        sub.callback(payload);
      } catch (err) {
        console.error(`Bus subscriber error on channel ${channelId}:`, err);
      }
    }
  }

  subscribe(channelId: string, callback: BusCallback): () => void {
    const channel = this.channels.get(channelId);
    if (!channel) {
      throw new Error(`Bus channel not registered: ${channelId}. Register before subscribing.`);
    }

    const id = this.nextSubId++;
    channel.subscribers.push({ callback, id });

    // retainLast: replay the last value to new subscriber
    if (channel.definition.retainLast && channel.hasPublished) {
      try {
        callback(channel.lastPayload);
      } catch (err) {
        console.error(`Bus subscriber replay error on channel ${channelId}:`, err);
      }
    }

    // Return unsubscribe function
    return () => {
      const idx = channel.subscribers.findIndex((s) => s.id === id);
      if (idx !== -1) {
        channel.subscribers.splice(idx, 1);
      }
    };
  }

  /** Returns the last published value, or undefined if never published. */
  getLastValue(channelId: string): unknown {
    const channel = this.channels.get(channelId);
    if (!channel) {
      throw new Error(`Bus channel not registered: ${channelId}`);
    }
    return channel.hasPublished ? channel.lastPayload : undefined;
  }

  /** Returns all registered channel IDs. */
  listChannels(): string[] {
    return Array.from(this.channels.keys());
  }

  private assertPayloadSize(channelId: string, payload: unknown): void {
    try {
      const json = JSON.stringify(payload);
      if (json && json.length > DEV_PAYLOAD_LIMIT) {
        console.warn(
          `Bus payload on "${channelId}" exceeds ${DEV_PAYLOAD_LIMIT} bytes (${json.length} bytes). ` +
            `Bus carries signals, not data — consider using a projection instead.`
        );
      }
    } catch {
      // Non-serialisable payloads are fine (e.g. undefined, functions) — skip check
    }
  }
}

/** Singleton bus instance for the application. */
export const bus = new MessageBus(import.meta.env.DEV);
