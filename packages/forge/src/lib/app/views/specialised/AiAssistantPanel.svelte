<script lang="ts">
  /**
   * AiAssistantPanel — placeholder panel for future AI assistant.
   *
   * Shows Urd branding, "AI Assistant" header, "Coming soon" message.
   * Chat-like interface with a disabled text input.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();
</script>

<div class="forge-ai-assistant">
  <div class="forge-ai-assistant__toolbar">
    <span class="forge-ai-assistant__title">AI Assistant</span>
  </div>

  <div class="forge-ai-assistant__content">
    <div class="forge-ai-assistant__brand">
      <span class="forge-ai-assistant__brand-mark">Urd</span>
      <span class="forge-ai-assistant__brand-sub">AI Assistant</span>
    </div>

    <div class="forge-ai-assistant__message">
      <p class="forge-ai-assistant__coming-soon">Coming soon</p>
      <p class="forge-ai-assistant__description">
        The Urd AI Assistant will help you author Schema Markdown,
        diagnose compilation issues, and explore your world design.
      </p>
    </div>
  </div>

  <div class="forge-ai-assistant__input-area">
    <input
      class="forge-ai-assistant__input"
      type="text"
      placeholder="Ask the assistant..."
      disabled
    />
  </div>
</div>

<style>
  .forge-ai-assistant {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-ai-assistant__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-ai-assistant__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-ai-assistant__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--forge-space-xl);
    padding: var(--forge-space-xl);
  }

  .forge-ai-assistant__brand {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--forge-space-xs);
  }

  .forge-ai-assistant__brand-mark {
    font-size: 28px;
    font-weight: 700;
    color: var(--forge-text-primary);
    letter-spacing: 0.08em;
  }

  .forge-ai-assistant__brand-sub {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .forge-ai-assistant__message {
    text-align: center;
    max-width: 280px;
  }

  .forge-ai-assistant__coming-soon {
    font-size: var(--forge-font-size-md);
    font-weight: 600;
    color: var(--forge-text-secondary);
    margin-bottom: var(--forge-space-sm);
  }

  .forge-ai-assistant__description {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    line-height: 1.5;
  }

  .forge-ai-assistant__input-area {
    padding: var(--forge-space-md);
    border-top: 1px solid var(--forge-border-zone);
    background: var(--forge-bg-secondary);
    flex-shrink: 0;
  }

  .forge-ai-assistant__input {
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .forge-ai-assistant__input::placeholder {
    color: var(--forge-text-muted);
  }
</style>
