<script lang="ts">
  /**
   * Breadcrumb — shows structural position within the current document.
   *
   * Parses backwards from the cursor line to find the nearest headings and
   * section labels, displaying a trail like:
   *   filename > # Location > ## Sequence > == section
   *
   * Each segment is clickable and scrolls the editor to that line.
   */

  interface BreadcrumbSegment {
    label: string;
    line: number;
    kind: 'file' | 'location' | 'sequence' | 'phase' | 'section';
  }

  interface Props {
    filePath: string | null;
    cursorLine: number;
    getDocText: () => string | null;
    onNavigate: (line: number) => void;
  }

  let { filePath, cursorLine, getDocText, onNavigate }: Props = $props();

  let segments: BreadcrumbSegment[] = $derived.by(() => {
    if (!filePath) return [];
    const result: BreadcrumbSegment[] = [];

    // File name segment
    const fileName = filePath.split('/').pop() || filePath;
    result.push({ label: fileName, line: 1, kind: 'file' });

    const text = getDocText();
    if (!text || cursorLine < 1) return result;

    const lines = text.split('\n');
    const limit = Math.min(cursorLine, lines.length);

    // Scan backwards from cursor to find enclosing structure
    let nearestLocation: BreadcrumbSegment | null = null;
    let nearestSequence: BreadcrumbSegment | null = null;
    let nearestPhase: BreadcrumbSegment | null = null;
    let nearestSection: BreadcrumbSegment | null = null;

    for (let i = limit - 1; i >= 0; i--) {
      const line = lines[i];
      const lineNum = i + 1;

      // Location heading: # Name
      if (!nearestLocation && /^#\s+(?!#)/.test(line)) {
        const match = line.match(/^#\s+(.+)/);
        if (match) {
          nearestLocation = { label: `# ${match[1].trim()}`, line: lineNum, kind: 'location' };
        }
      }

      // Sequence heading: ## Name
      if (!nearestSequence && /^##\s+(?!#)/.test(line)) {
        const match = line.match(/^##\s+(.+)/);
        if (match) {
          nearestSequence = { label: `## ${match[1].trim()}`, line: lineNum, kind: 'sequence' };
        }
      }

      // Phase heading: ### Name
      if (!nearestPhase && /^###\s+/.test(line)) {
        const match = line.match(/^###\s+(.+)/);
        if (match) {
          nearestPhase = { label: `### ${match[1].trim()}`, line: lineNum, kind: 'phase' };
        }
      }

      // Section label: == name
      if (!nearestSection && /^==\s+/.test(line)) {
        const match = line.match(/^==\s+(.+)/);
        if (match) {
          nearestSection = { label: `== ${match[1].trim()}`, line: lineNum, kind: 'section' };
        }
      }

      // If we've found a location heading, stop — that's the outermost container
      if (nearestLocation) break;
    }

    // Build trail in structural order
    if (nearestLocation) result.push(nearestLocation);
    if (nearestSequence) result.push(nearestSequence);
    if (nearestPhase) result.push(nearestPhase);
    if (nearestSection) result.push(nearestSection);

    return result;
  });
</script>

<div class="forge-breadcrumb" role="navigation" aria-label="Document structure">
  {#each segments as segment, i (segment.line)}
    {#if i > 0}
      <span class="forge-breadcrumb__separator">&#x25B8;</span>
    {/if}
    <button
      class="forge-breadcrumb__segment forge-breadcrumb__segment--{segment.kind}"
      onclick={() => onNavigate(segment.line)}
      title="Go to line {segment.line}"
    >{segment.label}</button>
  {/each}
</div>

<style>
  .forge-breadcrumb {
    display: flex;
    align-items: center;
    height: 22px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    gap: var(--forge-space-xs);
  }

  .forge-breadcrumb::-webkit-scrollbar {
    display: none;
  }

  .forge-breadcrumb__separator {
    color: var(--forge-text-muted);
    font-size: 8px;
    line-height: 1;
    flex-shrink: 0;
  }

  .forge-breadcrumb__segment {
    display: inline-block;
    padding: 0 2px;
    border: none;
    background: transparent;
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
    white-space: nowrap;
    border-radius: var(--forge-radius-sm);
    transition: background-color 0.1s, color 0.1s;
  }

  .forge-breadcrumb__segment:hover {
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }

  .forge-breadcrumb__segment--file {
    color: var(--forge-text-muted);
  }

  .forge-breadcrumb__segment--location {
    color: var(--forge-syntax-heading);
  }

  .forge-breadcrumb__segment--sequence {
    color: var(--forge-syntax-heading);
  }

  .forge-breadcrumb__segment--phase {
    color: var(--forge-syntax-heading);
  }

  .forge-breadcrumb__segment--section {
    color: var(--forge-syntax-section);
  }
</style>
