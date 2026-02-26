/**
 * Svelte 5 adapter for the message bus.
 * Auto-subscribe/unsubscribe with last-value replay.
 */

import { bus, type BusCallback } from './MessageBus';

/**
 * Subscribe to a bus channel within a Svelte component lifecycle.
 * Returns an unsubscribe function. Call in onMount or $effect.
 *
 * Usage in a Svelte 5 component:
 * ```svelte
 * <script lang="ts">
 *   import { subscribeToBus } from '$lib/framework/bus/useBusChannel';
 *   import { onMount } from 'svelte';
 *
 *   let lastEvent = $state<unknown>(undefined);
 *   let unsub: (() => void) | undefined;
 *
 *   onMount(() => {
 *     unsub = subscribeToBus('layout.changed', (payload) => {
 *       lastEvent = payload;
 *     });
 *     return () => unsub?.();
 *   });
 * </script>
 * ```
 */
export function subscribeToBus(channelId: string, callback: BusCallback): () => void {
  return bus.subscribe(channelId, callback);
}

/**
 * Publish to a bus channel.
 * Convenience re-export so components don't need to import bus directly.
 */
export function publishToBus(channelId: string, payload: unknown): void {
  bus.publish(channelId, payload);
}
