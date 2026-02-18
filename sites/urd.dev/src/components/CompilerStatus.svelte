<script lang="ts">
  import { onMount } from 'svelte';

  // ── Types ──────────────────────────────────────────────────────────────────

  interface TestEntry {
    name: string;
    status: 'pass' | 'fail' | 'skip';
    duration_ms: number | null;
  }

  interface Category {
    name: string;
    tests: number;
    passed: number;
    failed: number;
    test_names: TestEntry[];
  }

  interface Phase {
    name: string;
    diagnostic_range: string | null;
    tests: number;
    passed: number;
    failed: number;
    skipped: number;
    duration_ms: number | null;
    categories: Category[];
    diagnostics: string[];
  }

  interface Compliance {
    label: string;
    status: 'pass' | 'fail';
    note: string;
  }

  interface BenchmarkFile {
    name: string;
    source_bytes: number;
    output_bytes: number;
    total_ms: number;
    phases: {
      parse_ms: number;
      import_ms: number;
      link_ms: number;
      validate_ms: number;
      emit_ms: number;
    };
    success: boolean;
    diagnostic_count: number;
  }

  interface BenchmarkAggregate {
    total_source_bytes: number;
    total_output_bytes: number;
    total_ms: number;
    files_compiled: number;
    files_succeeded: number;
    avg_ms_per_file: number;
    bytes_per_ms: number;
  }

  interface Report {
    version: string;
    timestamp: string;
    compiler: { name: string; version: string; language: string };
    summary: {
      total_tests: number;
      passed: number;
      failed: number;
      skipped: number;
      duration_ms: number;
      pass_rate: number;
    };
    phases: Phase[];
    compliance: Compliance[];
    benchmarks: { files: BenchmarkFile[]; aggregate: BenchmarkAggregate };
  }

  // ── State ──────────────────────────────────────────────────────────────────

  let report: Report | null = $state(null);
  let loaded = $state(false);
  let activeTab: 'phases' | 'compliance' | 'benchmarks' = $state('phases');
  let expandedPhases: Set<string> = $state(new Set());

  // ── Derived ────────────────────────────────────────────────────────────────

  let timestamp = $derived(
    report
      ? new Date(report.timestamp).toLocaleDateString('en-GB', {
          day: 'numeric',
          month: 'short',
          year: 'numeric',
        })
      : '',
  );

  let passRateDisplay = $derived(
    report ? (report.summary.pass_rate * 100).toFixed(1) + '%' : '',
  );

  let durationDisplay = $derived(
    report
      ? report.summary.duration_ms >= 1000
        ? (report.summary.duration_ms / 1000).toFixed(1) + 's'
        : report.summary.duration_ms + 'ms'
      : '',
  );

  let phaseCount = $derived(report ? report.phases.length : 0);

  let allPassing = $derived(
    report ? report.summary.failed === 0 : false,
  );

  // ── Methods ────────────────────────────────────────────────────────────────

  function togglePhase(name: string) {
    const next = new Set(expandedPhases);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    expandedPhases = next;
  }

  function formatMs(ms: number): string {
    if (ms >= 1000) return (ms / 1000).toFixed(2) + 's';
    if (ms >= 1) return ms.toFixed(1) + 'ms';
    return (ms * 1000).toFixed(0) + 'us';
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    if (bytes >= 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return bytes + ' B';
  }

  function phasePassRate(phase: Phase): number {
    return phase.tests > 0 ? (phase.passed / phase.tests) * 100 : 100;
  }

  // ── Lifecycle ──────────────────────────────────────────────────────────────

  onMount(async () => {
    try {
      const res = await fetch('/compiler.json');
      if (res.ok) {
        const data = await res.json();
        if (data) report = data;
      }
    } catch {
      // Silently handle fetch errors — component shows empty state
    }
    loaded = true;
  });
</script>

<section class="cs">
  {#if loaded && report}
    <header class="cs-header">
      <span class="cs-label">Compiler <span class="cs-label-sep">·</span> Validation</span>
      <h2 class="cs-title">Test Suite</h2>
      <p class="cs-subtitle">
        {report.summary.total_tests} tests across 5 phases
        <span class="cs-sep">·</span>
        {timestamp}
      </p>
    </header>

    <!-- Summary stats -->
    <div class="cs-stats">
      <div class="cs-stat" class:cs-stat-pass={allPassing} class:cs-stat-fail={!allPassing}>
        <span class="cs-stat-value">{passRateDisplay}</span>
        <span class="cs-stat-label">Pass Rate</span>
      </div>
      <div class="cs-stat">
        <span class="cs-stat-value">{report.summary.total_tests}</span>
        <span class="cs-stat-label">Tests</span>
      </div>
      <div class="cs-stat">
        <span class="cs-stat-value">{durationDisplay}</span>
        <span class="cs-stat-label">Duration</span>
      </div>
      <div class="cs-stat">
        <span class="cs-stat-value">v{report.compiler.version}</span>
        <span class="cs-stat-label">{report.compiler.language}</span>
      </div>
    </div>

    <!-- Tabs -->
    <div class="cs-tabs" role="tablist">
      <button
        class="cs-tab"
        class:cs-tab-active={activeTab === 'phases'}
        role="tab"
        aria-selected={activeTab === 'phases'}
        onclick={() => (activeTab = 'phases')}
      >
        Phases
        <span class="cs-tab-count">{phaseCount}</span>
      </button>
      <button
        class="cs-tab"
        class:cs-tab-active={activeTab === 'compliance'}
        role="tab"
        aria-selected={activeTab === 'compliance'}
        onclick={() => (activeTab = 'compliance')}
      >
        Compliance
        <span class="cs-tab-count">{report.compliance.length}</span>
      </button>
      <button
        class="cs-tab"
        class:cs-tab-active={activeTab === 'benchmarks'}
        role="tab"
        aria-selected={activeTab === 'benchmarks'}
        onclick={() => (activeTab = 'benchmarks')}
      >
        Benchmarks
        {#if report.benchmarks.files.length > 0}
          <span class="cs-tab-count">{report.benchmarks.files.length}</span>
        {/if}
      </button>
    </div>

    <!-- Phases panel -->
    {#if activeTab === 'phases'}
      <div class="cs-panel" role="tabpanel">
        <div class="cs-phase-grid">
          {#each report.phases as phase}
            <div
              class="cs-phase"
              class:cs-phase-expanded={expandedPhases.has(phase.name)}
              class:cs-phase-clean={phase.failed === 0}
              class:cs-phase-failing={phase.failed > 0}
            >
              <button
                class="cs-phase-top"
                aria-expanded={expandedPhases.has(phase.name)}
                onclick={() => togglePhase(phase.name)}
              >
                <div class="cs-phase-row">
                  <span class="cs-phase-name">{phase.name}</span>
                  {#if phase.diagnostic_range}
                    <span class="cs-phase-diag">URD {phase.diagnostic_range}</span>
                  {/if}
                  <span class="cs-phase-counts">
                    <span class="cs-phase-passed">{phase.passed}</span><span class="cs-phase-slash">/{phase.tests}</span>
                    {#if phase.failed > 0}
                      <span class="cs-phase-failed">{phase.failed} ✗</span>
                    {/if}
                  </span>
                  <span class="cs-phase-chevron">{expandedPhases.has(phase.name) ? '▾' : '▸'}</span>
                </div>

                <div class="cs-phase-bar">
                  <div
                    class="cs-phase-bar-fill"
                    class:cs-bar-pass={phase.failed === 0}
                    class:cs-bar-fail={phase.failed > 0}
                    style="width: {phasePassRate(phase)}%"
                  ></div>
                </div>

                <div class="cs-phase-meta">
                  {#if phase.duration_ms !== null}
                    <span class="cs-phase-duration">{formatMs(phase.duration_ms)}</span>
                  {/if}
                  {#if phase.diagnostics.length > 0}
                    <span class="cs-phase-diag-count">{phase.diagnostics.length} diagnostics</span>
                  {/if}
                  <span class="cs-phase-cat-count">{phase.categories.length} categories</span>
                </div>
              </button>

              {#if expandedPhases.has(phase.name)}
                <div class="cs-phase-detail">
                  {#each phase.categories as cat}
                    <div class="cs-cat">
                      <div class="cs-cat-header">
                        <span class="cs-cat-name">{cat.name}</span>
                        <span class="cs-cat-count">{cat.passed}/{cat.tests}</span>
                      </div>
                      <div class="cs-cat-tests">
                        {#each cat.test_names as test}
                          <div
                            class="cs-test"
                            class:cs-test-pass={test.status === 'pass'}
                            class:cs-test-fail={test.status === 'fail'}
                            class:cs-test-skip={test.status === 'skip'}
                          >
                            <span class="cs-test-status"
                              >{test.status === 'pass'
                                ? '✓'
                                : test.status === 'fail'
                                  ? '✗'
                                  : '–'}</span
                            >
                            <span class="cs-test-name">{test.name}</span>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Compliance panel -->
    {#if activeTab === 'compliance'}
      <div class="cs-panel" role="tabpanel">
        <div class="cs-compliance-grid">
          {#each report.compliance as check}
            <div
              class="cs-check"
              class:cs-check-pass={check.status === 'pass'}
              class:cs-check-fail={check.status === 'fail'}
            >
              <span class="cs-check-icon"
                >{check.status === 'pass' ? '✓' : '✗'}</span
              >
              <div class="cs-check-body">
                <span class="cs-check-label">{check.label}</span>
                <span class="cs-check-note">{check.note}</span>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Benchmarks panel -->
    {#if activeTab === 'benchmarks'}
      <div class="cs-panel" role="tabpanel">
        {#if report.benchmarks.files.length === 0}
          <div class="cs-empty">
            <span class="cs-empty-text">No benchmark data available.</span>
          </div>
        {:else}
          <div class="cs-bench-summary">
            <div class="cs-bench-stat">
              <span class="cs-bench-stat-value">{report.benchmarks.aggregate.files_compiled}</span>
              <span class="cs-bench-stat-label">files</span>
            </div>
            <div class="cs-bench-stat">
              <span class="cs-bench-stat-value">{formatMs(report.benchmarks.aggregate.total_ms)}</span>
              <span class="cs-bench-stat-label">total</span>
            </div>
            <div class="cs-bench-stat">
              <span class="cs-bench-stat-value">{formatMs(report.benchmarks.aggregate.avg_ms_per_file)}</span>
              <span class="cs-bench-stat-label">avg/file</span>
            </div>
            <div class="cs-bench-stat">
              <span class="cs-bench-stat-value">{formatBytes(report.benchmarks.aggregate.bytes_per_ms)}/ms</span>
              <span class="cs-bench-stat-label">throughput</span>
            </div>
          </div>

          <div class="cs-bench-files">
            {#each report.benchmarks.files as file}
              <div class="cs-bench-file" class:cs-bench-file-ok={file.success} class:cs-bench-file-err={!file.success}>
                <div class="cs-bench-file-header">
                  <span class="cs-bench-file-name">{file.name}</span>
                  <span class="cs-bench-file-time">{formatMs(file.total_ms)}</span>
                </div>
                <div class="cs-bench-file-meta">
                  <span>{formatBytes(file.source_bytes)} in</span>
                  <span class="cs-bench-arrow">→</span>
                  <span>{formatBytes(file.output_bytes)} out</span>
                  {#if file.diagnostic_count > 0}
                    <span class="cs-bench-diags">{file.diagnostic_count} diag</span>
                  {/if}
                </div>
                <div class="cs-bench-phases">
                  {#each Object.entries(file.phases) as [phaseName, ms]}
                    <div class="cs-bench-phase">
                      <span class="cs-bench-phase-name">{phaseName.replace('_ms', '')}</span>
                      <div class="cs-bench-phase-bar-track">
                        <div
                          class="cs-bench-phase-bar-fill"
                          style="width: {file.total_ms > 0 ? (ms / file.total_ms) * 100 : 0}%"
                        ></div>
                      </div>
                      <span class="cs-bench-phase-ms">{formatMs(ms)}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {:else if loaded}
    <header class="cs-header">
      <span class="cs-label">Compiler <span class="cs-label-sep">·</span> Validation</span>
      <h2 class="cs-title">Test Suite</h2>
    </header>
    <div class="cs-empty">
      <span class="cs-empty-text">No test report available. Run <code>pnpm compiler:test</code> to generate one.</span>
    </div>
  {/if}
</section>

<style>
  /* ── Container ── */
  .cs {
    width: 100%;
  }

  /* ── Header ── */
  .cs-header {
    margin-bottom: 24px;
  }

  .cs-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    display: block;
    margin-bottom: 6px;
  }

  .cs-label-sep {
    color: var(--border-light);
    margin: 0 2px;
  }

  .cs-title {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .cs-subtitle {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  .cs-sep {
    color: var(--border-light);
    margin: 0 4px;
  }

  /* ── Stats row ── */
  .cs-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: 24px;
  }

  .cs-stat {
    background: var(--raise);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .cs-stat-value {
    font-family: var(--mono);
    font-size: clamp(22px, 3vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.02em;
    line-height: 1;
  }

  .cs-stat-pass .cs-stat-value {
    color: var(--green-light);
  }

  .cs-stat-fail .cs-stat-value {
    color: var(--rose);
  }

  .cs-stat-label {
    font-family: var(--display);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--faint);
  }

  /* ── Tabs ── */
  .cs-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 18px;
  }

  .cs-tab {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: color 0.15s ease, border-color 0.15s ease, background 0.15s ease;
  }

  .cs-tab:hover {
    color: var(--dim);
    border-color: var(--border-light);
  }

  .cs-tab-active {
    color: var(--amber-light);
    border-color: var(--amber-dark);
    background: rgba(204, 122, 58, 0.06);
  }

  .cs-tab-active:hover {
    color: var(--amber-light);
  }

  .cs-tab-count {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 500;
    color: var(--faint);
    background: var(--deep);
    padding: 2px 6px;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
  }

  .cs-tab-active .cs-tab-count {
    color: var(--amber-light);
    background: rgba(204, 122, 58, 0.12);
  }

  .cs-tab:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  /* ── Panels ── */
  .cs-panel {
    animation: fadeIn 0.25s ease-out;
  }

  /* ── Phase grid ── */
  .cs-phase-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .cs-phase {
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .cs-phase-clean {
    border-left: 3px solid var(--green-dim);
  }

  .cs-phase-failing {
    border-left: 3px solid var(--rose);
  }

  .cs-phase-top {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .cs-phase-top:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .cs-phase-top:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: -2px;
    border-radius: 6px;
  }

  .cs-phase-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .cs-phase-name {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--amber-light);
  }

  .cs-phase-diag {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    background: var(--deep);
    padding: 2px 7px;
    border-radius: 10px;
    letter-spacing: 0.03em;
  }

  .cs-phase-counts {
    margin-left: auto;
    display: flex;
    align-items: baseline;
    gap: 2px;
  }

  .cs-phase-passed {
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 600;
    color: var(--green-light);
  }

  .cs-phase-slash {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
  }

  .cs-phase-failed {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--rose);
    margin-left: 6px;
  }

  .cs-phase-chevron {
    font-size: 10px;
    color: var(--amber);
    margin-left: 4px;
    transition: color 0.15s ease;
  }

  .cs-phase-top:hover .cs-phase-chevron {
    color: var(--amber-light);
  }

  /* Progress bar */
  .cs-phase-bar {
    height: 2px;
    background: var(--deep);
    border-radius: 1px;
    overflow: hidden;
  }

  .cs-phase-bar-fill {
    height: 100%;
    border-radius: 1px;
    transition: width 0.4s ease;
  }

  .cs-bar-pass {
    background: var(--green);
  }

  .cs-bar-fail {
    background: var(--rose);
  }

  .cs-phase-meta {
    display: flex;
    gap: 10px;
  }

  .cs-phase-duration,
  .cs-phase-diag-count,
  .cs-phase-cat-count {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
  }

  /* ── Phase detail (expanded) ── */
  .cs-phase-detail {
    border-top: 1px solid var(--border);
    padding: 14px 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: fadeIn 0.25s ease-out;
  }

  .cs-cat {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .cs-cat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .cs-cat-name {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--dim);
  }

  .cs-cat-count {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
  }

  .cs-cat-tests {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .cs-test {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
  }

  .cs-test-status {
    font-size: 11px;
    width: 14px;
    text-align: center;
    flex-shrink: 0;
  }

  .cs-test-pass .cs-test-status {
    color: var(--green);
  }

  .cs-test-fail .cs-test-status {
    color: var(--rose);
  }

  .cs-test-skip .cs-test-status {
    color: var(--faint);
  }

  .cs-test-name {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cs-test-pass .cs-test-name {
    color: var(--dim);
  }

  .cs-test-fail .cs-test-name {
    color: var(--rose);
  }

  /* ── Compliance grid ── */
  .cs-compliance-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .cs-check {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px 18px;
  }

  .cs-check-icon {
    font-size: 14px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .cs-check-pass .cs-check-icon {
    color: var(--green-light);
    background: rgba(112, 176, 128, 0.1);
  }

  .cs-check-fail .cs-check-icon {
    color: var(--rose);
    background: rgba(204, 136, 136, 0.1);
  }

  .cs-check-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .cs-check-label {
    font-family: var(--display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .cs-check-note {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
    line-height: 1.5;
  }

  /* ── Benchmarks ── */
  .cs-bench-summary {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: 16px;
  }

  .cs-bench-stat {
    background: var(--raise);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .cs-bench-stat-value {
    font-family: var(--mono);
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    line-height: 1;
  }

  .cs-bench-stat-label {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--faint);
  }

  .cs-bench-files {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .cs-bench-file {
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .cs-bench-file-ok {
    border-left: 3px solid var(--green-dim);
  }

  .cs-bench-file-err {
    border-left: 3px solid var(--rose);
  }

  .cs-bench-file-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .cs-bench-file-name {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .cs-bench-file-time {
    font-family: var(--mono);
    font-size: 14px;
    font-weight: 600;
    color: var(--amber-light);
  }

  .cs-bench-file-meta {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .cs-bench-arrow {
    color: var(--border-light);
  }

  .cs-bench-diags {
    color: var(--amber);
    margin-left: 4px;
  }

  .cs-bench-phases {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .cs-bench-phase {
    display: grid;
    grid-template-columns: 56px 1fr 48px;
    align-items: center;
    gap: 8px;
  }

  .cs-bench-phase-name {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    text-align: right;
  }

  .cs-bench-phase-bar-track {
    height: 3px;
    background: var(--deep);
    border-radius: 2px;
    overflow: hidden;
  }

  .cs-bench-phase-bar-fill {
    height: 100%;
    background: var(--amber);
    border-radius: 2px;
  }

  .cs-bench-phase-ms {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    text-align: right;
  }

  /* ── Empty state ── */
  .cs-empty {
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 32px 24px;
    text-align: center;
  }

  .cs-empty-text {
    font-family: var(--mono);
    font-size: 13px;
    color: var(--faint);
  }

  .cs-empty-text :global(code) {
    font-family: var(--mono);
    color: var(--dim);
    background: var(--deep);
    padding: 2px 6px;
    border-radius: 4px;
  }

  /* ── Animation ── */
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @media (prefers-reduced-motion: reduce) {
    .cs-panel {
      animation: none;
    }

    .cs-phase-detail {
      animation: none;
    }
  }

  /* ── Responsive ── */
  @media (max-width: 980px) {
    .cs-phase-grid {
      grid-template-columns: 1fr;
    }

    .cs-compliance-grid {
      grid-template-columns: 1fr;
    }

    .cs-bench-files {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .cs-stats {
      grid-template-columns: repeat(2, 1fr);
    }

    .cs-bench-summary {
      grid-template-columns: repeat(2, 1fr);
    }

    .cs-tabs {
      flex-wrap: wrap;
    }
  }
</style>
