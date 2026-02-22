<script lang="ts">
  import { onMount, tick } from 'svelte';

  interface PresentationSection {
    id: string;
    numeral: string;
    label: string;
    shortLabel: string;
  }

  const sections: PresentationSection[] = [
    { id: 'pres-welcome', numeral: '', label: 'Welcome', shortLabel: 'Welcome' },
    { id: 'pres-tradition', numeral: 'I', label: 'A Long Tradition', shortLabel: 'Tradition' },
    { id: 'pres-gap', numeral: 'II', label: 'The Gap', shortLabel: 'The Gap' },
    { id: 'pres-collaborator', numeral: 'III', label: 'A New Collaborator', shortLabel: 'Collaborator' },
    { id: 'pres-idea', numeral: 'IV', label: 'The Idea', shortLabel: 'The Idea' },
    { id: 'pres-proof', numeral: 'V', label: 'The Proof', shortLabel: 'The Proof' },
    { id: 'pres-pair', numeral: 'VI', label: 'Two Halves of a Whole', shortLabel: 'Urd + Wyrd' },
    { id: 'pres-how', numeral: 'VII', label: 'How It Works', shortLabel: 'How It Works' },
    { id: 'pres-different', numeral: 'VIII', label: 'What Makes It Different', shortLabel: 'Difference' },
    { id: 'pres-limits', numeral: 'IX', label: 'Where It Breaks', shortLabel: 'Limits' },
    { id: 'pres-status', numeral: 'X', label: 'Where We Are', shortLabel: 'Status' },
    { id: 'pres-closing', numeral: '', label: 'Take a Look Around', shortLabel: 'Explore' },
  ];

  let isOpen = $state(false);
  let hasBeenOpened = $state(false);
  let currentSection = $state(0);
  let savedScrollY = $state(0);
  let navHeight = $state(0);
  let reducedMotion = $state(false);

  let scrollProgress = $state(0);
  let isNavigating = $state(false);
  let navTimer: ReturnType<typeof setTimeout> | null = null;

  // Set to true to re-enable the audio companion player
  const audioEnabled = false;

  let audioEl: HTMLAudioElement | undefined = $state(undefined);
  let isPlaying = $state(false);
  let audioProgress = $state(0);
  let audioDuration = $state(0);
  let audioCurrentTime = $state(0);
  const playbackRates = [1, 1.1, 1.2, 1.3, 1.35] as const;
  let playbackRateIndex = $state(0);
  let playbackRate = $derived(playbackRates[playbackRateIndex]);

  let wrapperEl: HTMLElement | undefined = $state(undefined);
  let headerEl: HTMLElement | undefined = $state(undefined);

  let observer: IntersectionObserver | null = null;
  let revealObserver: IntersectionObserver | null = null;

  function measureNav(): void {
    const nav = document.getElementById('nav-bar');
    if (nav) navHeight = nav.offsetHeight;
  }

  function updateScrollProgress(): void {
    if (!isOpen) return;
    const first = document.getElementById(sections[0].id);
    if (!first) return;
    const headerHeight = headerEl?.offsetHeight ?? 48;
    const offset = navHeight + headerHeight;
    const start = first.getBoundingClientRect().top + window.scrollY - offset;
    const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
    if (maxScroll <= start) { scrollProgress = 1; return; }
    const progress = (window.scrollY - start) / (maxScroll - start);
    scrollProgress = Math.max(0, Math.min(1, progress));

    // Sync section indicator when scrolled to the very end
    if (scrollProgress >= 0.98 && !isNavigating) {
      currentSection = sections.length - 1;
    }
  }

  function open(): void {
    savedScrollY = window.scrollY;
    hasBeenOpened = true;
    isOpen = true;

    // Enter presentation mode — drives hero collapse, nav simplification, content hiding
    document.documentElement.setAttribute('data-presentation', 'open');
    window.dispatchEvent(new CustomEvent('urd:presentation-changed', { detail: { open: true } }));

    const btn = document.getElementById('presentation-trigger');
    if (btn) btn.setAttribute('aria-expanded', 'true');

    // After transition, scroll to presentation
    if (wrapperEl) {
      const onEnd = async () => {
        wrapperEl?.removeEventListener('transitionend', onEnd);
        headerEl?.scrollIntoView({ behavior: reducedMotion ? 'instant' : 'smooth', block: 'start' });
        headerEl?.focus();
        setupObserver();
        await tick();
        setupRevealObserver();
      };
      wrapperEl.addEventListener('transitionend', onEnd, { once: true });
      // Fallback if transition doesn't fire (e.g. reduced motion)
      if (reducedMotion) {
        setTimeout(onEnd, 50);
      }
    }
  }

  function close(): void {
    if (audioEl && isPlaying) {
      audioEl.pause();
      isPlaying = false;
    }
    isOpen = false;
    teardownObserver();

    const btn = document.getElementById('presentation-trigger');
    if (btn) btn.setAttribute('aria-expanded', 'false');

    if (wrapperEl) {
      const onEnd = () => {
        wrapperEl?.removeEventListener('transitionend', onEnd);
        // Exit presentation mode after collapse — restores hero, nav, content
        document.documentElement.removeAttribute('data-presentation');
        window.dispatchEvent(new CustomEvent('urd:presentation-changed', { detail: { open: false } }));
        window.scrollTo({ top: savedScrollY, behavior: 'instant' });
        const triggerBtn = document.getElementById('presentation-trigger');
        triggerBtn?.focus();
      };
      wrapperEl.addEventListener('transitionend', onEnd, { once: true });
      if (reducedMotion) {
        setTimeout(onEnd, 50);
      }
    }
  }

  function setupObserver(): void {
    teardownObserver();
    const headerHeight = headerEl?.offsetHeight ?? 48;
    observer = new IntersectionObserver(
      (entries) => {
        if (isNavigating) return;
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const index = sections.findIndex(s => s.id === entry.target.id);
            if (index !== -1) currentSection = index;
          }
        }
      },
      { rootMargin: `-${navHeight + headerHeight}px 0px -50% 0px` }
    );
    for (const section of sections) {
      const el = document.getElementById(section.id);
      if (el) observer.observe(el);
    }
  }

  function teardownObserver(): void {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
    if (revealObserver) {
      revealObserver.disconnect();
      revealObserver = null;
    }
  }

  function setupRevealObserver(): void {
    if (reducedMotion) {
      // Show everything immediately
      for (const el of document.querySelectorAll('.pres-reveal')) {
        el.classList.add('visible');
      }
      return;
    }
    revealObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            entry.target.classList.add('visible');
            revealObserver?.unobserve(entry.target);
          }
        }
      },
      { rootMargin: '0px 0px -60px 0px', threshold: 0.01 }
    );
    for (const el of document.querySelectorAll('.pres-reveal')) {
      revealObserver.observe(el);
    }
  }

  function toggleAudio(): void {
    if (!audioEl) return;
    if (isPlaying) {
      audioEl.pause();
      isPlaying = false;
    } else {
      audioEl.play();
      isPlaying = true;
    }
  }

  function onAudioTimeUpdate(): void {
    if (!audioEl) return;
    audioCurrentTime = audioEl.currentTime;
    audioProgress = audioDuration > 0 ? audioEl.currentTime / audioDuration : 0;
  }

  function onAudioMetadata(): void {
    if (!audioEl) return;
    audioDuration = audioEl.duration;
  }

  function onAudioEnded(): void {
    isPlaying = false;
    audioProgress = 1;
  }

  function seekAudio(e: MouseEvent): void {
    if (!audioEl || audioDuration <= 0) return;
    const bar = e.currentTarget as HTMLElement;
    const rect = bar.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    audioEl.currentTime = ratio * audioDuration;
  }

  function skipAudio(seconds: number): void {
    if (!audioEl || audioDuration <= 0) return;
    audioEl.currentTime = Math.max(0, Math.min(audioDuration, audioEl.currentTime + seconds));
  }

  function seekAudioKey(e: KeyboardEvent): void {
    if (!audioEl || audioDuration <= 0) return;
    if (e.key === 'ArrowRight') skipAudio(10);
    else if (e.key === 'ArrowLeft') skipAudio(-10);
  }

  function cyclePlaybackRate(): void {
    playbackRateIndex = (playbackRateIndex + 1) % playbackRates.length;
    if (audioEl) audioEl.playbackRate = playbackRate;
  }

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function navigateToSection(index: number): void {
    if (index < 0 || index >= sections.length) return;
    const el = document.getElementById(sections[index].id);
    if (!el) return;

    // Lock navigation so the observer doesn't fight the scroll
    isNavigating = true;
    currentSection = index;
    if (navTimer) clearTimeout(navTimer);
    navTimer = setTimeout(() => { isNavigating = false; }, 800);

    const headerHeight = headerEl?.offsetHeight ?? 48;
    const offset = navHeight + headerHeight + 16;
    const top = el.getBoundingClientRect().top + window.scrollY - offset;
    window.scrollTo({ top, behavior: reducedMotion ? 'instant' : 'smooth' });
  }

  $effect(() => {
    const el = audioEl;
    if (!el) return;
    el.playbackRate = playbackRate;
    el.addEventListener('timeupdate', onAudioTimeUpdate);
    el.addEventListener('loadedmetadata', onAudioMetadata);
    el.addEventListener('ended', onAudioEnded);
    return () => {
      el.removeEventListener('timeupdate', onAudioTimeUpdate);
      el.removeEventListener('loadedmetadata', onAudioMetadata);
      el.removeEventListener('ended', onAudioEnded);
    };
  });

  onMount(() => {
    reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    measureNav();

    const scrollHandler = () => { measureNav(); updateScrollProgress(); };
    window.addEventListener('scroll', scrollHandler, { passive: true });
    window.addEventListener('resize', measureNav, { passive: true });

    const openHandler = () => open();
    const closeHandler = () => close();
    window.addEventListener('urd:open-presentation', openHandler);
    window.addEventListener('urd:close-presentation', closeHandler);

    return () => {
      window.removeEventListener('scroll', scrollHandler);
      window.removeEventListener('resize', measureNav);
      window.removeEventListener('urd:open-presentation', openHandler);
      window.removeEventListener('urd:close-presentation', closeHandler);
      teardownObserver();
    };
  });
</script>

<div
  class="pres-wrapper"
  class:open={isOpen}
  id="presentation-container"
  role="region"
  aria-label="Interactive project presentation"
  bind:this={wrapperEl}
>
  <div class="pres-outer">
    {#if hasBeenOpened}
      {#if audioEnabled}
        <audio
          bind:this={audioEl}
          src="/audio/introduction.m4a"
          preload="none"
        ></audio>
      {/if}

      <!-- Sticky header — outside the overflow container so sticky works -->
      <div
        class="pres-header"
        style="top: {navHeight}px"
        bind:this={headerEl}
        tabindex="-1"
      >
        <div class="pres-header-inner">
          <button
            class="pres-nav-btn"
            disabled={currentSection === 0}
            onclick={() => navigateToSection(currentSection - 1)}
            aria-label="Previous section"
          >
            <span aria-hidden="true">◂</span> Prev
          </button>

          <span class="pres-section-indicator" aria-live="polite">
            {#if sections[currentSection].numeral}
              <span class="pres-section-numeral">{sections[currentSection].numeral}</span>
              <span class="pres-section-sep" aria-hidden="true">·</span>
            {/if}
            {sections[currentSection].shortLabel}
          </span>

          {#if audioEnabled}
            <div class="pres-audio-slot">
              {#if audioDuration > 0}
                <button class="pres-audio-skip" onclick={() => skipAudio(-10)} aria-label="Back 10 seconds">
                  <span aria-hidden="true">◂◂</span>
                </button>
                <button class="pres-audio-toggle" onclick={toggleAudio} aria-label={isPlaying ? 'Pause' : 'Play'}>
                  <span aria-hidden="true">{isPlaying ? '❚❚' : '▶'}</span>
                </button>
                <button class="pres-audio-skip" onclick={() => skipAudio(10)} aria-label="Forward 10 seconds">
                  <span aria-hidden="true">▸▸</span>
                </button>
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <div class="pres-audio-bar" onclick={seekAudio} onkeydown={seekAudioKey} role="slider" tabindex="0" aria-label="Audio progress" aria-valuenow={Math.round(audioProgress * 100)} aria-valuemin={0} aria-valuemax={100}>
                  <div class="pres-audio-bar-fill" style="width: {audioProgress * 100}%"></div>
                </div>
                <span class="pres-audio-time">{formatTime(audioCurrentTime)}</span>
                <button class="pres-audio-rate" onclick={cyclePlaybackRate} aria-label="Playback speed {playbackRate}x">
                  {playbackRate}x
                </button>
              {/if}
            </div>
          {/if}

          <button
            class="pres-nav-btn"
            disabled={currentSection === sections.length - 1}
            onclick={() => navigateToSection(currentSection + 1)}
            aria-label="Next section"
          >
            Next <span aria-hidden="true">▸</span>
          </button>
        </div>
        <div class="pres-progress" role="progressbar" aria-valuenow={Math.round(scrollProgress * 100)} aria-valuemin={0} aria-valuemax={100}>
          <div class="pres-progress-fill" style="width: {scrollProgress * 100}%"></div>
        </div>
      </div>

      <!-- Content -->
      <div class="pres-content">

        <!-- ═══ WELCOME ═══ -->
        <section class="pres-section pres-section-welcome" id="pres-welcome">
          <span class="pres-welcome-eyebrow pres-reveal">An experiment in progress</span>
          <h2 class="pres-welcome-heading pres-reveal">Welcome to Urd</h2>
          <p class="pres-welcome-subtitle pres-reveal">
            This project asks a question: can you write a formal specification for
            interactive worlds and hand it to AI to build? What follows is
            the lineage, the thesis, and an honest account of where we are.
          </p>
          {#if audioEnabled}
            <div class="pres-listen-controls pres-reveal">
              {#if audioDuration > 0}
                <button class="pres-listen-skip" onclick={() => skipAudio(-10)} aria-label="Back 10 seconds">
                  <span class="pres-listen-skip-icon" aria-hidden="true">↺</span>
                  <span class="pres-listen-skip-label">10s</span>
                </button>
              {/if}
              <button class="pres-listen-btn" onclick={toggleAudio} aria-label={isPlaying ? 'Pause narration' : 'Listen to narration'}>
                <span class="pres-listen-icon" aria-hidden="true">{isPlaying ? '❚❚' : '▶'}</span>
              </button>
              {#if audioDuration > 0}
                <button class="pres-listen-skip" onclick={() => skipAudio(10)} aria-label="Forward 10 seconds">
                  <span class="pres-listen-skip-icon" aria-hidden="true">↻</span>
                  <span class="pres-listen-skip-label">10s</span>
                </button>
              {/if}
            </div>
            <div class="pres-listen-scrub" class:pres-listen-scrub-visible={audioDuration > 0}>
              <span class="pres-listen-time">{formatTime(audioCurrentTime)}</span>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div class="pres-listen-bar" onclick={seekAudio} onkeydown={seekAudioKey} role="slider" tabindex="0" aria-label="Audio progress" aria-valuenow={Math.round(audioProgress * 100)} aria-valuemin={0} aria-valuemax={100}>
                <div class="pres-listen-bar-fill" style="width: {audioProgress * 100}%"></div>
              </div>
              <span class="pres-listen-time">−{formatTime(audioDuration - audioCurrentTime)}</span>
            </div>
            <span class="pres-listen-label pres-reveal">{isPlaying ? 'Listening' : audioCurrentTime > 0 ? 'Resume' : 'Listen along'}{#if audioDuration > 0}
              <button class="pres-listen-rate" onclick={cyclePlaybackRate} aria-label="Playback speed {playbackRate}x">{playbackRate}x</button>
            {/if}</span>
          {/if}
          <span class="pres-welcome-hint pres-reveal" aria-hidden="true">Take your time ↓</span>
        </section>

        <!-- ═══ I — A LONG TRADITION ═══ -->
        <section class="pres-section" id="pres-tradition">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">I</span>
          </div>
          <h2 class="pres-heading pres-reveal">A long tradition</h2>
          <p class="pres-reveal">
            People have been building interactive worlds with text for decades,
            and the tools they have created are remarkable — each one solving a
            real part of the puzzle.
          </p>

          <div class="pres-timeline pres-reveal">
            <div class="pres-timeline-entry">
              <div class="pres-timeline-marker"></div>
              <div class="pres-timeline-body">
                <span class="pres-timeline-era">1970s–90s</span>
                <span class="pres-timeline-title">MUDs and IF</span>
                <p>
                  Multi-User Dungeons proved that text worlds could be rich, spatial, and
                  multiplayer. Parser-based IF languages followed — ZIL at Infocom,
                  <span class="hl">TADS</span> in 1988,
                  <span class="hl">Inform</span> in 1993 — each building increasingly
                  sophisticated world models: rooms, objects, containment, rules, paired
                  with natural-language parsers.
                </p>
              </div>
            </div>

            <div class="pres-timeline-entry">
              <div class="pres-timeline-marker"></div>
              <div class="pres-timeline-body">
                <span class="pres-timeline-era">2009</span>
                <span class="pres-timeline-title">Twine</span>
                <p>
                  Twine democratised branching narrative. Anyone could create a
                  hypertext story with no programming at all. It opened the door
                  for writers, educators, and artists. But its world model is
                  passages and links — no objects, no space, no state beyond variables.
                </p>
              </div>
            </div>

            <div class="pres-timeline-entry">
              <div class="pres-timeline-marker"></div>
              <div class="pres-timeline-body">
                <span class="pres-timeline-era">2016</span>
                <span class="pres-timeline-title">ink</span>
                <p>
                  inkle's <span class="hl">ink</span> cracked the middleware
                  problem for dialogue. An elegant scripting language that compiles to
                  JSON and runs inside Unity, Unreal, or the browser. It powers
                  games like <em>80 Days</em> and <em>Heaven's Vault</em>. But ink is
                  dialogue — it has no spatial model, no inventory, no world simulation.
                </p>
              </div>
            </div>

            <div class="pres-timeline-entry">
              <div class="pres-timeline-marker pres-timeline-marker-now"></div>
              <div class="pres-timeline-body">
                <span class="pres-timeline-era">Today</span>
                <span class="pres-timeline-title">The current landscape</span>
                <p>
                  Yarn Spinner and articy:draft handle narrative design.
                  Unity and Godot handle simulation. Each is powerful within
                  its domain. But the world's <em>story</em> lives in one system, its
                  <em>space</em> in another, its <em>rules</em> in a third — stitched
                  together with custom glue code, every time.
                </p>
              </div>
            </div>
          </div>
        </section>

        <!-- ═══ II — THE GAP ═══ -->
        <section class="pres-section" id="pres-gap">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">II</span>
          </div>
          <h2 class="pres-heading pres-reveal">The gap</h2>
          <p class="pres-reveal">
            Inform came closest. It unified space, objects, rules, and narrative in a
            single system, and it did it thirty years ago. But the world it describes
            is tightly coupled to the Inform runtime. Handing an Inform world to
            Unity, to Godot, to a browser, or to an AI means working around the
            format, not with it. (Inform 10 has made strides here, but the world
            model and the execution engine remain deeply intertwined.)
          </p>
          <p class="pres-reveal">
            ink solved the portability problem, but only for dialogue. It compiles to a clean JSON
            format that any engine can consume. But it deliberately stops at conversation —
            no rooms, no objects, no containment.
          </p>
          <p class="pres-reveal">
            The combination has not been done: <span class="hl">a portable, structured
            data format</span>, like ink's JSON contract, <span class="hl">but for
            entire worlds</span>, like Inform's model. A format that describes space, objects,
            characters, rules, <em>and</em> narrative in one file that any runtime can execute
            without custom glue code.
          </p>
          <p class="pres-aside pres-reveal">
            That is the hypothesis this project is testing.
          </p>
        </section>

        <!-- ═══ III — A NEW COLLABORATOR ═══ -->
        <section class="pres-section" id="pres-collaborator">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">III</span>
          </div>
          <h2 class="pres-heading pres-reveal">A new collaborator at the table</h2>
          <p class="pres-reveal">
            There is another reason this matters now in a way it did not a decade ago.
          </p>
          <p class="pres-reveal">
            The tools in that lineage were built for a world where a single author, or a
            small team, typed every word, placed every object, wrote every rule by hand.
            That world is changing. <span class="hl">AI is becoming a creative
            collaborator</span> — not a replacement for human vision, but an amplifier of it.
            A writer who once spent hours hand-placing furniture in forty rooms can now
            describe what they want and let an AI fill in the details. A designer sketching
            a quest line can have an AI generate the supporting NPCs, their dialogue, their
            behavioural rules — all consistent with the world's existing structure.
          </p>
          <p class="pres-reveal">
            But here is the problem: most existing formats were not designed to be read
            by machines in any meaningful way. Twine stores its state in HTML passages.
            Inform's world model is embedded in natural-language source code. Game engine
            scenes are serialised into opaque binary formats. An AI can interact with these
            systems, but it is working <em>around</em> the format, not <em>with</em> it.
          </p>

          <div class="pres-callout pres-reveal">
            <span class="pres-callout-label">The insight</span>
            <p>
              If a world is described as <span class="hl">typed, structured,
              unambiguous data</span> — where every entity has a defined type, every
              property has a schema, every rule has explicit conditions and effects —
              then an AI does not need to guess what anything means. The format itself
              is a formal contract. An AI can read it, reason about it, generate
              content that conforms to it, and validate its own output against it.
              Not as a bolted-on feature. As an inherent property of the format.
            </p>
          </div>

          <p class="pres-reveal">
            This does not mean AI is required. Every Urd world works perfectly without it.
            But when an AI <em>is</em> part of the creative process — and increasingly it
            will be — the world it is helping to build should be described in a language
            it can actually understand. Not prose. Not markup hacks. A schema.
          </p>

          <p class="pres-reveal">
            This project is itself the first test of that thesis. The specifications,
            this site, and the compiler were built with AI as the primary
            workforce — iterating against the spec, challenging its own output, and
            failing in instructive ways. The compiler was built entirely from
            six engineering briefs without amending them once. It is an experiment
            in spec-driven AI development as much as it is a schema design.
          </p>
        </section>

        <!-- ═══ IV — THE IDEA ═══ -->
        <section class="pres-section" id="pres-idea">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">IV</span>
          </div>
          <h2 class="pres-heading pres-reveal">The idea</h2>
          <p class="pres-reveal">
            Urd starts from a simple premise: <span class="hl">what if you could describe
            an entire interactive world as structured data?</span>
          </p>
          <p class="pres-reveal">
            Not code. Not a script tied to an engine. Just a clear, typed description of
            what exists, where things are, what the rules are, and what can happen. A
            description that any runtime — whether a browser, a game engine, a text terminal,
            or an AI — could pick up and execute.
          </p>
          <p class="pres-reveal">
            A universal contract for interactive worlds.
          </p>
        </section>

        <!-- ═══ V — THE PROOF ═══ -->
        <section class="pres-section" id="pres-proof">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">V</span>
          </div>
          <h2 class="pres-heading pres-reveal">The proof</h2>
          <p class="pres-reveal">
            Ideas are cheap. Here is a concrete one. The Monty Hall problem — three
            doors, a car, two goats, and a host who knows the secret — in a single
            file. The frontmatter defines what the world <em>is</em>. The body
            defines what the player <em>does</em>. This is a working test fixture
            that compiles today.
          </p>

          <div class="pres-code pres-reveal">
            <div class="pres-code-header">
              <span class="pres-code-filename">monty-hall.urd.md</span>
              <span class="pres-code-badge">Working Fixture</span>
            </div>
            <pre class="pres-code-block"><code>---
world:
  name: monty-hall
  start: stage
types:
  Door [interactable]:
    <span class="code-hidden">~</span>prize: enum(goat, car)
    revealed: bool = false
entities:
  <span class="code-entity">@door_1</span>: Door &#123; prize: "goat" &#125;
  <span class="code-entity">@door_2</span>: Door &#123; prize: "goat" &#125;
  <span class="code-entity">@door_3</span>: Door &#123; prize: "car" &#125;
  <span class="code-entity">@host</span>: Door
---

# Stage

[<span class="code-entity">@door_1</span>, <span class="code-entity">@door_2</span>, <span class="code-entity">@door_3</span>]

## The Game

### Choose

* Pick a door -&gt; any Door

### Reveal (auto)

rule monty_reveals:
  actor: <span class="code-entity">@host</span> action reveal
  selects door from [<span class="code-entity">@door_1</span>, <span class="code-entity">@door_2</span>, <span class="code-entity">@door_3</span>]
    where door.prize == goat
  <span class="code-effect">&gt;</span> reveal door.prize

### Switch

== switch

* Switch doors -&gt; any Door
  ? <span class="code-entity">@door_1</span>.revealed == false
* Stay with your choice

The host opens the final door.</code></pre>
            <a class="pres-code-try" href="/playground#code=LS0tCndvcmxkOgogIG5hbWU6IG1vbnR5LWhhbGwKICBzdGFydDogc3RhZ2UKdHlwZXM6CiAgRG9vciBbaW50ZXJhY3RhYmxlXToKICAgIH5wcml6ZTogZW51bShnb2F0LCBjYXIpCiAgICByZXZlYWxlZDogYm9vbCA9IGZhbHNlCmVudGl0aWVzOgogIEBkb29yXzE6IERvb3IgeyBwcml6ZTogImdvYXQiIH0KICBAZG9vcl8yOiBEb29yIHsgcHJpemU6ICJnb2F0IiB9CiAgQGRvb3JfMzogRG9vciB7IHByaXplOiAiY2FyIiB9CiAgQGhvc3Q6IERvb3IKLS0tCgojIFN0YWdlCgpbQGRvb3JfMSwgQGRvb3JfMiwgQGRvb3JfM10KCiMjIFRoZSBHYW1lCgojIyMgQ2hvb3NlCgoqIFBpY2sgYSBkb29yIC0+IGFueSBEb29yCgojIyMgUmV2ZWFsIChhdXRvKQoKcnVsZSBtb250eV9yZXZlYWxzOgogIGFjdG9yOiBAaG9zdCBhY3Rpb24gcmV2ZWFsCiAgc2VsZWN0cyBkb29yIGZyb20gW0Bkb29yXzEsIEBkb29yXzIsIEBkb29yXzNdCiAgICB3aGVyZSBkb29yLnByaXplID09IGdvYXQKICA-IHJldmVhbCBkb29yLnByaXplCgojIyMgU3dpdGNoCgo9PSBzd2l0Y2gKCiogU3dpdGNoIGRvb3JzIC0+IGFueSBEb29yCiAgPyBAZG9vcl8xLnJldmVhbGVkID09IGZhbHNlCiogU3RheSB3aXRoIHlvdXIgY2hvaWNlCgpUaGUgaG9zdCBvcGVucyB0aGUgZmluYWwgZG9vci4K">Try in Playground <span aria-hidden="true">▸</span></a>
          </div>

          <p class="pres-reveal">
            The <span class="code-hidden">~</span> before <code>prize</code> means
            it is hidden — the player cannot see it. The <code>(auto)</code> beats
            run without player input. The rule constrains the host: it can only
            reveal a door that hides a goat.
          </p>

          <div class="pres-callout pres-reveal">
            <span class="pres-callout-label">What to notice</span>
            <p>
              No probability is specified anywhere. Run this world 10,000 times and
              switching wins two thirds of the time. The 2/3 advantage is not
              coded — it <span class="hl">emerges from the structure</span>. That is
              what declarative world description means in practice.
            </p>
          </div>
        </section>

        <!-- ═══ VI — TWO HALVES ═══ -->
        <section class="pres-section" id="pres-pair">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">VI</span>
          </div>
          <h2 class="pres-heading pres-reveal">Two halves of a whole</h2>
          <p class="pres-reveal">
            The project has two parts, named from Norse mythology. They are
            complementary — one defines, the other executes.
          </p>

          <div class="pres-pair-grid pres-reveal">
            <div class="pres-pair-card pres-pair-urd">
              <h3>Urd</h3>
              <span class="pres-pair-etymology">Old Norse Ur&#240;r — "that which has become"</span>
              <p>
                The schema. The definition layer. Urd is how you describe what a world
                <em>is</em>: its entities, locations, rules, and narrative structures.
                It is the contract between writers and machines.
              </p>
            </div>
            <div class="pres-pair-card pres-pair-wyrd">
              <h3>Wyrd</h3>
              <span class="pres-pair-etymology">Old English — "fate, what comes to pass"</span>
              <p>
                The runtime. The execution layer. Wyrd takes a world definition and
                brings it to life — evaluating conditions, resolving actions, advancing
                state. It is destiny unfolding.
              </p>
            </div>
          </div>

          <div class="pres-callout pres-reveal">
            <span class="pres-callout-label">In practice</span>
            <p>
              Writers describe worlds in a clean markdown syntax. The compiler produces a
              <span class="pres-gold">.urd.json</span> file — the contract. The
              <span class="pres-purple">Wyrd</span> runtime reads that contract and
              simulates the world: evaluating conditions, applying effects, producing
              events. Everything the player actually sees and types is handled by a
              replaceable <span class="hl">adapter layer</span> — a parser, a choice UI,
              a graphical renderer, an AI agent. The writer never sees JSON. The runtime
              never sees prose. The adapter never changes the world definition.
            </p>
          </div>
        </section>

        <!-- ═══ VII — HOW IT WORKS ═══ -->
        <section class="pres-section" id="pres-how">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">VII</span>
          </div>
          <h2 class="pres-heading pres-reveal">How it works</h2>
          <p class="pres-reveal">
            Writers author worlds using <span class="hl">Schema Markdown</span>, a
            syntax designed to feel like writing prose. The entire vocabulary is seven symbols.
          </p>

          <div class="pres-symbols pres-reveal">
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">@</span>
              <span class="pres-symbol-label">Entity</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">?</span>
              <span class="pres-symbol-label">Condition</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">></span>
              <span class="pres-symbol-label">Effect</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">*</span>
              <span class="pres-symbol-label">Choice</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">+</span>
              <span class="pres-symbol-label">Sticky</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">&#8594;</span>
              <span class="pres-symbol-label">Jump</span>
            </div>
            <div class="pres-symbol">
              <span class="pres-symbol-glyph">//</span>
              <span class="pres-symbol-label">Comment</span>
            </div>
          </div>

          <p class="pres-reveal">
            That is it. A character is <span class="pres-gold">@halvard</span>. A condition
            checks state. An effect changes it. Choices branch. Jumps navigate. If the syntax
            forces a writer to touch type definitions or JSON, the tooling has failed.
          </p>

          <div class="pres-flow pres-reveal">
            <span class="pres-flow-step pres-flow-write">.urd.md</span>
            <span class="pres-flow-arrow" aria-hidden="true">&#8594;</span>
            <span class="pres-flow-step pres-flow-compile">compiler</span>
            <span class="pres-flow-arrow" aria-hidden="true">&#8594;</span>
            <span class="pres-flow-step pres-flow-compile">.urd.json</span>
            <span class="pres-flow-arrow" aria-hidden="true">&#8594;</span>
            <span class="pres-flow-step pres-flow-run">Wyrd runtime</span>
          </div>
          <div class="pres-flow pres-flow-secondary pres-reveal">
            <span class="pres-flow-ghost"></span>
            <span class="pres-flow-ghost"></span>
            <span class="pres-flow-step pres-flow-compile">compiler</span>
            <span class="pres-flow-arrow" aria-hidden="true">&#8594;</span>
            <span class="pres-flow-step pres-flow-analysis">FactSet</span>
            <span class="pres-flow-arrow" aria-hidden="true">&#8594;</span>
            <span class="pres-flow-step pres-flow-analysis">analysis + tooling</span>
          </div>
        </section>

        <!-- ═══ VIII — WHAT MAKES IT DIFFERENT ═══ -->
        <section class="pres-section" id="pres-different">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">VIII</span>
          </div>
          <h2 class="pres-heading pres-reveal">What makes it different</h2>
          <p class="pres-reveal">
            <span class="hl">Declarative, not imperative.</span> You describe what the
            world <em>is</em>, not what it <em>does</em>. A door is locked. A guard reveals
            information under certain conditions. The runtime figures out when and how.
            Outcomes emerge from structure, not scripted sequences — as the Monty Hall
            example demonstrates.
          </p>
          <p class="pres-reveal">
            <span class="hl">One spatial primitive.</span> A room holds a sword. A chest
            holds a sword. A player's inventory holds a sword. It is all containment. One
            mechanism replaces three separate systems. Moving, picking up, and storing are
            the same operation.
          </p>
          <p class="pres-reveal">
            <span class="hl">The world is not the interface.</span> Every interactive world
            system eventually absorbs its presentation layer — verb synonyms, text
            composition, failure messages — until the world model <em>is</em> the UI. Urd draws a
            permanent architectural boundary: the schema describes what exists, the runtime
            simulates what happens, and everything the player sees or types belongs to a
            replaceable adapter layer. That boundary is enforced by a governance document,
            not by good intentions.
          </p>
          <p class="pres-reveal">
            <span class="hl">Engine agnostic.</span> Because the <span class="pres-gold">.urd.json</span>
            contract carries no rendering instructions — no pixel coordinates, no audio references,
            no UI layouts — a Unity plugin, a Godot addon, a browser, or a plain terminal can
            all consume the same file.
          </p>
          <p class="pres-reveal">
            <span class="hl">AI native.</span> Every element is typed and unambiguous.
            An AI reading an Urd world does not need to guess what anything means. It is a
            formal contract, not documentation. But every world works perfectly without AI.
          </p>
          <p class="pres-reveal">
            <span class="hl">Deterministic and testable.</span> Same world, same seed,
            same actions, same result. The runtime produces no output except in response to
            explicit action calls, and every state change is a typed event. The compiler
            extracts a <span class="hl">FactSet</span> — a flat, queryable graph of every
            relationship in the world — and uses it to check for unreachable locations,
            contradictory conditions, and dead-end dialogue before the runtime ever loads
            the file. Run a world ten thousand times and assert on the probability
            distribution. That is what a typed graph makes possible.
          </p>
        </section>

        <!-- ═══ IX — WHERE IT BREAKS ═══ -->
        <section class="pres-section" id="pres-limits">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">IX</span>
          </div>
          <h2 class="pres-heading pres-reveal">Where it breaks</h2>
          <p class="pres-reveal">
            A declarative schema can describe a locked door, a hidden prize, a
            conversation that branches on trust. But the interesting parts of
            interactive fiction have always been the parts that are <em>not</em>
            built in — the custom mechanic, the unexpected verb, the interaction
            nobody anticipated.
          </p>
          <p class="pres-reveal">
            This is a well-known problem. It has been called "database-driven IF"
            and it has been tried before, going back to the 1980s. Systems that
            delegate all behaviour to a runtime tend to produce worlds that feel
            generic. The things players remember most are almost never the
            standard library.
          </p>
          <p class="pres-reveal">
            Urd's answer — not yet built, but designed — is a
            <span class="hl">lambda extension host</span>: a way for the runtime
            to delegate specific behaviours to custom logic under strict constraints —
            read-only state in, a list of effects out. The schema stays declarative.
            The lambda is sandboxed imperative logic that the runtime supervises.
            Whether that boundary is in the right place is one of the things this
            project exists to find out.
          </p>
          <p class="pres-aside pres-reveal">
            If your first reaction is scepticism, good. That is the correct reaction
            to an unproven claim. The spec is public. The test cases are defined. The
            failures will be too.
          </p>
        </section>

        <!-- ═══ X — WHERE WE ARE ═══ -->
        <section class="pres-section" id="pres-status">
          <div class="pres-divider pres-reveal">
            <span class="pres-numeral">X</span>
          </div>
          <h2 class="pres-heading pres-reveal">Where we are</h2>
          <p class="pres-reveal">
            The specification is complete and formalised. A <span class="hl">PEG grammar</span>
            with 75 rules defines what valid input looks like. A <span class="hl">JSON Schema</span>
            with 9 sub-schemas defines what valid output looks like. An
            <span class="hl">Architectural Boundaries</span> governance document defines what
            belongs in the schema, what belongs in the runtime, and what is permanently
            excluded from both — including a formal failure contract, deterministic
            trigger semantics, and a five-question boundary test for evaluating every
            proposed change. Six engineering briefs specify every data structure, every
            diagnostic code, every phase contract.
          </p>
          <p class="pres-reveal">
            The <span class="hl">compiler is built</span>. Five phases — PARSE, IMPORT,
            LINK, VALIDATE, EMIT — implemented in Rust with <span class="hl">554 tests
            and a 100% pass rate</span>. Five canonical fixtures compile to valid
            <span class="pres-gold">.urd.json</span>. The design documents were not
            amended once during implementation. Human specification, AI implementation,
            machine verification.
          </p>
          <p class="pres-reveal">
            The <span class="hl">v1 completion gate is closed</span>. All 32
            acceptance criteria — nine compiler requirements, eight static analysis
            checks, eight FactSet requirements, and seven specification consistency
            items — verified and documented. The next milestone is
            <span class="hl">alpha</span>: the <span class="hl">Wyrd reference
            runtime</span> loading <span class="pres-gold">.urd.json</span>, executing
            the canonical test cases, and proving the system works end-to-end — compile the
            Monty Hall problem, run it ten thousand times, verify the switching advantage
            converges to two-thirds.
          </p>
          <p class="pres-reveal">
            This site, <span class="pres-gold">urd.dev</span>, is the development journal.
            Every design document, architectural decision, and progress update is published
            here as it happens — including the parts that do not work. The test dashboard
            on the homepage shows every test, every phase, every benchmark — live numbers,
            not claims.
          </p>
        </section>

        <!-- ═══ CLOSING ═══ -->
        <section class="pres-section pres-section-closing" id="pres-closing">
          <h2 class="pres-heading pres-heading-closing pres-reveal">Take a look around</h2>
          <p class="pres-closing-text pres-reveal">
            You are welcome here. Read the specifications, explore the design documents,
            or simply watch the experiment unfold.
          </p>
          <div class="pres-closing-links pres-reveal">
            <button class="pres-closing-link pres-closing-primary" onclick={close} aria-label="Close presentation">
              <span aria-hidden="true">✕</span>
            </button>
          </div>
        </section>

      </div>
    {/if}
  </div>
</div>

<style>
  /* ── Wrapper animation ── */
  .pres-wrapper {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 0.5s ease-out;
  }

  .pres-wrapper.open {
    grid-template-rows: 1fr;
  }

  .pres-outer {
    overflow: clip;
    min-height: 0;
    min-width: 0;
  }

  /* ── Scroll reveal ── */
  .pres-reveal {
    opacity: 0;
    transform: translateY(16px);
    transition: opacity 0.5s ease, transform 0.5s ease;
  }

  .pres-reveal:global(.visible) {
    opacity: 1;
    transform: translateY(0);
  }

  /* Stagger siblings within a section */
  .pres-reveal:nth-child(2) { transition-delay: 0.06s; }
  .pres-reveal:nth-child(3) { transition-delay: 0.12s; }
  .pres-reveal:nth-child(4) { transition-delay: 0.18s; }
  .pres-reveal:nth-child(5) { transition-delay: 0.24s; }
  .pres-reveal:nth-child(6) { transition-delay: 0.30s; }
  .pres-reveal:nth-child(7) { transition-delay: 0.36s; }

  @media (prefers-reduced-motion: reduce) {
    .pres-wrapper {
      transition-duration: 0.01ms;
    }

    .pres-progress-fill,
    .pres-listen-btn {
      animation: none;
    }

    .pres-reveal {
      opacity: 1;
      transform: none;
      transition: none;
    }
  }

  /* ── Sticky header ── */
  .pres-header {
    position: sticky;
    z-index: 5;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    outline: none;
  }

  .pres-header-inner {
    max-width: 1160px;
    margin: 0 auto;
    padding: 10px 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    position: relative;
  }

  .pres-nav-btn {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 500;
    color: var(--dim);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 12px;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .pres-nav-btn:hover:not(:disabled) {
    color: var(--text);
    border-color: var(--border-light);
  }

  .pres-nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .pres-nav-btn:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 1px;
  }

  .pres-section-indicator {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    font-family: var(--display);
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--gold-dim);
    pointer-events: none;
  }

  .pres-section-numeral {
    color: var(--faint);
  }

  .pres-section-sep {
    color: var(--border-light);
    margin: 0 4px;
  }

  .pres-audio-slot {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
  }

  .pres-audio-toggle {
    font-size: 11px;
    color: var(--faint);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
    transition: color 0.15s ease;
  }

  .pres-audio-toggle:hover { color: var(--text); }

  .pres-audio-skip {
    font-size: 13px;
    color: var(--faint);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
    transition: color 0.15s ease;
  }

  .pres-audio-skip:hover { color: var(--text); }

  .pres-audio-bar {
    width: 60px;
    height: 3px;
    background: var(--border);
    border-radius: 2px;
    cursor: pointer;
    position: relative;
  }

  .pres-audio-bar:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .pres-audio-bar-fill {
    height: 100%;
    background: var(--gold-dark);
    border-radius: 2px;
  }

  .pres-audio-time {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    min-width: 28px;
  }

  .pres-audio-rate {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 4px;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .pres-audio-rate:hover {
    color: var(--text);
    border-color: var(--border-light);
  }

  /* ── Progress bar ── */
  .pres-progress {
    height: 2px;
    background: var(--border);
  }

  .pres-progress-fill {
    height: 100%;
    background: var(--gold-dark);
    box-shadow: 0 0 6px color-mix(in srgb, var(--gold) 30%, transparent);
    animation: pres-bar-breathe 3s ease-in-out infinite;
  }

  @keyframes pres-bar-breathe {
    0%, 100% { box-shadow: 0 0 4px color-mix(in srgb, var(--gold) 20%, transparent); }
    50% { box-shadow: 0 0 8px color-mix(in srgb, var(--gold) 45%, transparent); }
  }

  /* ── Content ── */
  .pres-content {
    max-width: 680px;
    margin: 0 auto;
    padding: 0 32px 64px;
    overflow-x: hidden;
  }

  .pres-section {
    padding: 80px 0;
    border-top: 1px solid var(--border);
    scroll-margin-top: 120px;
  }

  .pres-section:first-child {
    border-top: none;
  }

  /* ── Welcome section ── */
  .pres-section-welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    min-height: 60vh;
    padding: 96px 0;
    border-top: none;
  }

  .pres-welcome-eyebrow {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 400;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--gold-dim);
    margin-bottom: 20px;
  }

  .pres-welcome-heading {
    font-family: var(--display);
    font-size: clamp(28px, 5vw, 40px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.02em;
    line-height: 1.1;
    margin-bottom: 20px;
  }

  .pres-welcome-subtitle {
    font-family: var(--body);
    font-size: clamp(16px, 2.5vw, 19px);
    color: var(--dim);
    line-height: 1.7;
    max-width: 480px;
    margin-bottom: 40px;
  }

  .pres-listen-controls {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 8px;
  }

  .pres-listen-skip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--faint);
    padding: 4px;
    transition: color 0.15s ease;
  }

  .pres-listen-skip:hover { color: var(--text); }

  .pres-listen-skip-icon {
    font-size: 18px;
  }

  .pres-listen-skip-label {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: 0.05em;
  }

  .pres-listen-btn {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: 1px solid var(--gold-dark);
    background: none;
    color: var(--gold);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.2s ease, color 0.2s ease, background 0.2s ease;
    animation: pres-listen-pulse 3s ease-in-out infinite;
  }

  @keyframes pres-listen-pulse {
    0%, 100% { box-shadow: 0 0 0 0 transparent; }
    50% { box-shadow: 0 0 12px 2px color-mix(in srgb, var(--gold) 20%, transparent); }
  }

  .pres-listen-btn:hover {
    border-color: var(--gold-dim);
    color: var(--gold-light);
    background: color-mix(in srgb, var(--gold) 4%, transparent);
  }

  .pres-listen-btn:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .pres-listen-scrub {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 280px;
    max-height: 0;
    margin-bottom: 0;
    opacity: 0;
    overflow: hidden;
    transition: max-height 0.4s ease, opacity 0.4s ease, margin-bottom 0.4s ease;
  }

  .pres-listen-scrub-visible {
    max-height: 24px;
    margin-bottom: 12px;
    opacity: 1;
  }

  .pres-listen-bar {
    flex: 1;
    height: 3px;
    background: var(--border);
    border-radius: 2px;
    cursor: pointer;
    position: relative;
  }

  .pres-listen-bar:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .pres-listen-bar-fill {
    height: 100%;
    background: var(--gold-dark);
    border-radius: 2px;
  }

  .pres-listen-time {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    min-width: 32px;
    text-align: center;
  }

  .pres-listen-label {
    font-family: var(--display);
    font-size: 12px;
    color: var(--faint);
    letter-spacing: 0.08em;
    margin-bottom: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pres-listen-rate {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
    cursor: pointer;
    letter-spacing: 0;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .pres-listen-rate:hover {
    color: var(--text);
    border-color: var(--border-light);
  }

  .pres-welcome-hint {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 400;
    color: var(--faint);
    letter-spacing: 0.08em;
    opacity: 0.6;
  }

  /* ── Section dividers ── */
  .pres-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 32px;
  }

  .pres-divider::before,
  .pres-divider::after {
    content: '';
    height: 1px;
    width: 48px;
    background: linear-gradient(to right, transparent, var(--border-light), transparent);
  }

  .pres-numeral {
    font-family: var(--display);
    font-size: 14px;
    font-weight: 300;
    color: var(--gold-dim);
    letter-spacing: 0.05em;
  }

  /* ── Typography ── */
  .pres-heading {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 28px;
  }

  .pres-content p {
    font-family: var(--body);
    font-size: 18px;
    color: var(--dim);
    line-height: 1.75;
    margin-bottom: 1.5rem;
  }

  .pres-content p:last-child {
    margin-bottom: 0;
  }

  .hl {
    color: var(--text);
    font-weight: 500;
  }

  .pres-gold {
    color: var(--gold);
  }

  .pres-purple {
    color: var(--purple-light);
  }

  .pres-aside {
    color: var(--faint);
    font-style: italic;
  }

  /* ── Timeline ── */
  .pres-timeline {
    position: relative;
    margin: 36px 0 12px;
    padding-left: 28px;
  }

  .pres-timeline::before {
    content: '';
    position: absolute;
    left: 5px;
    top: 8px;
    bottom: 8px;
    width: 1px;
    background: linear-gradient(to bottom, var(--gold-dark), var(--border), var(--purple-dim));
  }

  .pres-timeline-entry {
    position: relative;
    padding-bottom: 36px;
  }

  .pres-timeline-entry:last-child {
    padding-bottom: 0;
  }

  .pres-timeline-marker {
    position: absolute;
    left: -28px;
    top: 6px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: var(--bg);
    border: 1.5px solid var(--gold-dim);
  }

  .pres-timeline-marker-now {
    border-color: var(--purple);
    box-shadow: 0 0 6px rgba(176, 144, 221, 0.2);
  }

  .pres-timeline-era {
    display: block;
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--faint);
    margin-bottom: 4px;
  }

  .pres-timeline-title {
    display: block;
    font-family: var(--display);
    font-size: 17px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 8px;
  }

  .pres-timeline-body p {
    font-size: 16px;
    margin-bottom: 0;
  }

  /* ── Callouts ── */
  .pres-callout {
    border-left: 2px solid var(--border-light);
    padding: 20px 24px;
    margin: 32px 0;
    background: var(--raise);
    border-radius: 0 4px 4px 0;
  }

  .pres-callout-label {
    display: block;
    font-family: var(--display);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--faint);
    margin-bottom: 8px;
  }

  .pres-callout p {
    font-size: 16px;
    margin-bottom: 0;
  }

  /* ── Pair cards ── */
  .pres-pair-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin: 36px 0;
  }

  .pres-pair-card {
    padding: 20px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--raise);
    position: relative;
    border-left: 3px solid var(--border);
  }

  .pres-pair-urd {
    border-left-color: var(--gold);
  }

  .pres-pair-wyrd {
    border-left-color: var(--purple);
  }

  .pres-pair-card h3 {
    font-family: var(--display);
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .pres-pair-urd h3 {
    color: var(--gold);
  }

  .pres-pair-wyrd h3 {
    color: var(--purple-light);
  }

  .pres-pair-etymology {
    display: block;
    font-family: var(--body);
    font-size: 13px;
    color: var(--faint);
    font-style: italic;
    margin-bottom: 14px;
    letter-spacing: 0.02em;
  }

  .pres-pair-card p {
    font-size: 15px;
    line-height: 1.6;
    margin-bottom: 0;
  }

  /* ── Code block ── */
  .pres-code {
    margin: 32px 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    background: var(--deep);
  }

  .pres-code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--raise);
    border-bottom: 1px solid var(--border);
  }

  .pres-code-filename {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--dim);
  }

  .pres-code-badge {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--gold-dim);
    background: color-mix(in srgb, var(--gold) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--gold) 20%, transparent);
    padding: 2px 8px;
    border-radius: 3px;
  }

  .pres-code-block {
    padding: 20px 16px;
    margin: 0;
    font-family: var(--mono);
    font-size: 13px;
    line-height: 1.65;
    color: var(--dim);
    overflow-x: auto;
    white-space: pre;
    tab-size: 2;
  }

  .pres-code-block code {
    font-family: inherit;
    font-size: inherit;
  }

  .pres-code-try {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 16px;
    font-family: var(--display);
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.06em;
    color: var(--gold-dim);
    text-decoration: none;
    border-top: 1px solid var(--border);
    transition: color 0.15s ease, background 0.15s ease;
  }

  .pres-code-try:hover {
    color: var(--gold);
    background: color-mix(in srgb, var(--gold) 4%, transparent);
  }

  .code-hidden {
    color: var(--gold);
    font-weight: 600;
  }

  .code-entity {
    color: var(--gold-dim);
  }

  .code-effect {
    color: var(--purple);
  }

  /* ── Symbols ── */
  .pres-symbols {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin: 36px 0;
    flex-wrap: wrap;
  }

  .pres-symbol {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-width: 48px;
  }

  .pres-symbol-glyph {
    font-family: var(--mono);
    font-size: 1.3rem;
    color: var(--gold);
    background: var(--deep);
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .pres-symbol-label {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--faint);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  /* ── Pipeline flow ── */
  .pres-flow {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin: 36px 0;
    flex-wrap: wrap;
  }

  .pres-flow-step {
    padding: 6px 14px;
    border-radius: 4px;
    font-family: var(--mono);
    font-size: 13px;
    letter-spacing: 0.02em;
  }

  .pres-flow-write {
    background: color-mix(in srgb, var(--blue) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--blue) 25%, transparent);
    color: var(--blue-light);
  }

  .pres-flow-compile {
    background: color-mix(in srgb, var(--gold) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--gold) 20%, transparent);
    color: var(--gold-dim);
  }

  .pres-flow-run {
    background: color-mix(in srgb, var(--purple) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--purple) 20%, transparent);
    color: var(--purple);
  }

  .pres-flow-secondary {
    margin-top: 6px;
    opacity: 0.7;
  }

  .pres-flow-ghost {
    width: 0;
    padding: 0;
    margin: 0;
    visibility: hidden;
  }

  .pres-flow-analysis {
    background: color-mix(in srgb, var(--green) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--green) 25%, transparent);
    color: var(--green-light);
  }

  .pres-flow-arrow {
    color: var(--faint);
    font-size: 16px;
  }

  /* ── Closing ── */
  .pres-section-closing {
    text-align: center;
    padding: 80px 0 96px;
  }

  .pres-heading-closing {
    text-align: center;
  }

  .pres-closing-text {
    text-align: center;
    max-width: 480px;
    margin: 0 auto 2rem;
  }

  .pres-closing-links {
    display: flex;
    gap: 12px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .pres-closing-link {
    font-size: 28px;
    line-height: 1;
    color: var(--faint);
    background: none;
    border: 1px solid var(--border);
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    cursor: pointer;
    transition: color 0.2s ease, border-color 0.2s ease, background 0.2s ease;
  }

  .pres-closing-link:hover {
    color: var(--text);
    border-color: var(--border-light);
    background: color-mix(in srgb, var(--text) 6%, transparent);
  }

  .pres-closing-link:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  /* ── Responsive ── */
  @media (max-width: 980px) {
    .pres-pair-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .pres-header-inner {
      padding: 8px 18px;
      gap: 8px;
    }

    .pres-nav-btn {
      font-size: 12px;
      padding: 4px 8px;
    }

    .pres-section-indicator {
      font-size: 11px;
    }

    .pres-symbols {
      gap: 10px;
    }

    .pres-code-block {
      font-size: 11px;
      padding: 16px 12px;
    }
  }

  @media (max-width: 840px) {
    .pres-audio-bar { width: 40px; }
    .pres-audio-time { display: none; }
    .pres-audio-rate { display: none; }
  }

  @media (max-width: 720px) {
    .pres-audio-bar { display: none; }
    .pres-audio-skip { display: none; }
  }

  @media (max-width: 520px) {
    .pres-content {
      padding-left: 18px;
      padding-right: 18px;
    }

    .pres-flow {
      gap: 6px;
    }

    .pres-flow-step {
      font-size: 11px;
      padding: 4px 10px;
    }
  }
</style>