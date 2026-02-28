/**
 * PlaybackService — spec-complete interactive fiction runtime that simulates
 * world interaction using compiled urdJson data.
 *
 * Supports: dialogue (sticky/one-shot, subchoices, goto, on_exhausted,
 * description), all effect types (set, move, destroy, reveal), conditions
 * (property, container, exhaustion), actions (entity/type-targeted),
 * sequences/phases, rules, and container model.
 *
 * When @urd/wyrd-core is built, this implementation will be replaced
 * while keeping the same public interface.
 */

import { bus } from '$lib/framework/bus/MessageBus';
import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
import type {
  UrdWorld,
  UrdLocation,
  UrdEffect,
  UrdAction,
  UrdSequence,
  UrdSequencePhase,
  UrdRule,
  FactSet,
} from '$lib/app/compiler/types';
import {
  type PlaybackState,
  type PlaybackEvent,
  type PlaybackEventType,
  type PlaybackEntity,
  type PlaybackExitInfo,
  type PlaybackInteraction,
  type PlaybackPickup,
  type PlaybackAction,
  type CoverageData,
  type NarrativeLine,
  type NarrativeLineKind,
  type RuntimeEntity,
  type DialogueNode,
  type DialogueChoice,
  type DialogueEffect,
  type DialogueChoiceView,
  createInitialPlaybackState,
  createInitialCoverage,
} from './playback-types';

// ===== Helpers =====

const fmtLoc = (id: string): string =>
  id.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());

class PlaybackServiceImpl {
  state: PlaybackState = $state(createInitialPlaybackState());
  events: PlaybackEvent[] = $state([]);
  coverage: CoverageData = $state(createInitialCoverage());

  private world: UrdWorld | null = null;
  private factSet: FactSet | null = null;
  private locationMap: Map<string, UrdLocation> = new Map();
  private dialogue: Record<string, DialogueNode> = {};
  private entityState: Map<string, RuntimeEntity> = new Map();
  private dialogueState: { dialogueId: string; subChoices: DialogueChoice[] | null } | null = null;
  private visitedLocations: Set<string> = new Set();
  private traversedExits: Set<string> = new Set();
  /** Dialogue IDs that are goto targets — should not auto-start on location entry. */
  private gotoTargets: Set<string> = new Set();
  /** Used one-shot choice IDs — filtered from available choices. */
  private usedChoices: Set<string> = new Set();
  /** Exhausted section IDs — all choices used or unavailable. */
  private exhaustedSections: Set<string> = new Set();
  /** Properties revealed by reveal effects (entity.property keys). */
  private revealedProperties: Set<string> = new Set();
  /** Destroyed entity IDs. */
  private destroyedEntities: Set<string> = new Set();
  /** Active sequence state. */
  private activeSequenceId: string | null = null;
  private activePhaseIndex = 0;
  /** Rules that have fired this turn (prevents infinite loops). */
  private firedRulesThisTurn: Set<string> = new Set();
  private eventCounter = 0;
  private narrativeCounter = 0;
  private unsubscribers: (() => void)[] = [];

  /** Call after bus channels are registered (during bootstrap). */
  init(): void {
    this.unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        if (this.state.status === 'playing') {
          this.reset();
        }
      }),
    );
  }

  loadWorld(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');

    if (!urdJson || urdJson.locations.length === 0) {
      this.state = { ...createInitialPlaybackState(), status: 'error' };
      return;
    }

    this.world = urdJson;
    this.factSet = factSet ?? null;
    this.locationMap.clear();
    this.entityState.clear();
    this.dialogueState = null;
    this.visitedLocations.clear();
    this.traversedExits.clear();
    this.usedChoices.clear();
    this.exhaustedSections.clear();
    this.revealedProperties.clear();
    this.destroyedEntities.clear();
    this.activeSequenceId = null;
    this.activePhaseIndex = 0;
    this.firedRulesThisTurn.clear();
    this.eventCounter = 0;
    this.narrativeCounter = 0;
    this.events = [];

    for (const loc of urdJson.locations) {
      this.locationMap.set(loc.id, loc);
    }

    // Parse dialogue tree and collect goto targets
    this.dialogue = {};
    this.gotoTargets.clear();
    if (urdJson.dialogue) {
      for (const [id, node] of Object.entries(urdJson.dialogue)) {
        this.dialogue[id] = node as DialogueNode;
      }
      this.collectGotoTargets();
    }

    // Initialise mutable entity state
    this.initEntityState();

    // Initialise coverage totals
    const sectionIds = new Set<string>();
    const choiceIds = new Set<string>();
    if (factSet) {
      for (const c of factSet.choices) {
        sectionIds.add(c.section);
        choiceIds.add(c.choice_id);
      }
    }
    this.coverage = {
      locations: { visited: [], total: urdJson.locations.length },
      sections: { visited: [], total: sectionIds.size },
      choices: { visited: [], total: choiceIds.size },
      exits: { visited: [], total: this.countTotalExits() },
    };

    this.state = createInitialPlaybackState();
    this.state.status = 'playing';

    this.emitEvent('world_loaded', `World loaded: ${urdJson.world?.name ?? 'Untitled'}`);

    // Start at world.start or first location
    const startId = urdJson.world?.start ?? urdJson.locations[0]?.id;
    if (startId) {
      this.enterLocation(startId);
    }

    // Activate entry sequence if specified
    if (urdJson.world?.entry && urdJson.sequences?.[urdJson.world.entry]) {
      this.activateSequence(urdJson.world.entry);
    }
  }

  reset(): void {
    this.loadWorld();
  }

  moveTo(locationId: string): void {
    if (this.state.status !== 'playing' || this.state.gameOver) return;
    if (!this.locationMap.has(locationId)) return;

    // Check if exit is blocked
    if (this.state.currentLocation) {
      const loc = this.locationMap.get(this.state.currentLocation.id);
      const exit = loc?.exits.find((e) => e.target === locationId);
      if (exit?.condition && !this.evalCond(exit.condition)) {
        const msg = exit.blocked_message || 'The way is blocked.';
        this.pushNarrative('blocked', msg);
        this.emitEvent('condition', `Exit blocked: ${msg}`);
        this.publishState();
        return;
      }

      // Apply exit effects
      if (exit?.effects) {
        this.applyEffects(exit.effects as DialogueEffect[]);
      }

      // Track exit coverage
      const dir = exit?.direction ?? '';
      const exitKey = `${this.state.currentLocation.id}/${dir}`;
      if (!this.traversedExits.has(exitKey)) {
        this.traversedExits.add(exitKey);
        this.coverage.exits = {
          ...this.coverage.exits,
          visited: [...this.coverage.exits.visited, exitKey],
        };
      }
    }

    this.state.turnCount++;
    this.dialogueState = null;
    this.firedRulesThisTurn.clear();
    this.emitEvent('move', `Moved to: ${locationId}`);
    this.enterLocation(locationId);

    // Evaluate rules triggered by location entry
    this.evaluateRules(`enter ${locationId}`);
    this.checkSequenceAdvance();
  }

  pickUp(entityId: string): void {
    if (this.state.status !== 'playing' || this.state.gameOver) return;

    const ent = this.entityState.get(entityId);
    if (!ent) return;

    const name = this.eName(entityId);
    this.moveEntityTo(entityId, 'player');

    this.state.turnCount++;
    this.pushNarrative('player_action', `Picked up the ${name}.`);
    this.emitEvent('effect', `Picked up: ${entityId}`);
    this.firedRulesThisTurn.clear();
    this.evaluateRules('always');
    this.refreshScene();
    this.publishState();
  }

  enterDialogue(dialogueId: string): void {
    if (this.state.status !== 'playing' || this.state.gameOver) return;

    const dlg = this.dialogue[dialogueId];
    if (!dlg) {
      this.emitEvent('condition', `Dialogue not found: ${dialogueId}`);
      return;
    }

    if (dlg.conditions && !this.evalConds(dlg.conditions)) {
      this.emitEvent('condition', `Dialogue conditions not met: ${dialogueId}`);
      return;
    }

    this.dialogueState = { dialogueId, subChoices: null };
    this.emitEvent('dialogue', `Entered: ${dialogueId}`);

    // Show description (narration) before prompt
    if (dlg.description) {
      this.pushNarrative('narration', dlg.description);
    }

    if (dlg.prompt) {
      this.pushNarrative('dlg_prompt', dlg.prompt.text, this.eName(dlg.prompt.speaker));
    }

    // Check if section is already exhausted
    if (this.isSectionExhausted(dialogueId)) {
      this.handleExhaustedSection(dialogueId);
      return;
    }

    this.refreshScene();
    this.publishState();
  }

  chooseDialogue(choiceIndex: number): void {
    if (this.state.status !== 'playing' || !this.dialogueState || this.state.gameOver) return;

    const choices = this.getAvailableChoices();
    if (choiceIndex < 0 || choiceIndex >= choices.length) return;

    const choice = choices[choiceIndex];

    // Check conditions
    if (choice.conditions && !this.evalChoiceConds(choice.conditions)) return;

    this.state.turnCount++;
    this.firedRulesThisTurn.clear();
    this.pushNarrative('player_action', choice.label);
    this.emitEvent('dialogue', `Player chose: "${choice.label}"`);

    // Track one-shot choice usage
    if (!choice.sticky) {
      this.usedChoices.add(choice.id);
    }

    // Track coverage
    if (!this.coverage.choices.visited.includes(choice.id)) {
      this.coverage.choices = {
        ...this.coverage.choices,
        visited: [...this.coverage.choices.visited, choice.id],
      };
    }

    // Apply effects
    if (choice.effects) {
      this.applyEffects(choice.effects);
    }

    // Show response
    if (choice.response) {
      this.pushNarrative('dlg_response', choice.response.text, this.eName(choice.response.speaker));
      this.emitEvent('dialogue', `${this.eName(choice.response.speaker)}: "${choice.response.text}"`);
    }

    // Navigate: goto > subchoices > sticky loop > end
    if (choice.goto) {
      if (choice.goto === 'end') {
        this.endGame();
        return;
      }
      this.navigateToSection(choice.goto);
    } else if (choice.choices && choice.choices.length > 0) {
      this.dialogueState = { dialogueId: this.dialogueState.dialogueId, subChoices: choice.choices };
    } else if (choice.sticky && this.dialogueState.dialogueId) {
      // Stay in current dialogue — loop back
      // Check if the section is now exhausted after this choice
      this.updateExhaustion(this.dialogueState.dialogueId);
    } else {
      this.dialogueState = null;
    }

    // Check section exhaustion for current dialogue
    if (this.dialogueState) {
      const currentId = this.dialogueState.dialogueId;
      if (this.isSectionExhausted(currentId)) {
        this.handleExhaustedSection(currentId);
        return;
      }
    }

    this.evaluateRules('always');
    this.checkSequenceAdvance();
    this.refreshScene();
    this.publishState();
  }

  endDialogue(): void {
    if (!this.dialogueState) return;

    this.dialogueState = null;
    this.pushNarrative('system', 'You step back.');
    this.emitEvent('dialogue', 'Ended dialogue');
    this.refreshScene();
    this.publishState();
  }

  performAction(actionId: string): void {
    if (this.state.status !== 'playing' || this.state.gameOver) return;
    if (!this.world?.actions) return;

    const action = this.world.actions[actionId];
    if (!action) return;

    // Check conditions
    if (action.conditions && !this.evalConds(action.conditions)) return;

    this.state.turnCount++;
    this.firedRulesThisTurn.clear();
    const desc = action.description ?? actionId;
    this.pushNarrative('player_action', desc);
    this.emitEvent('effect', `Action: ${actionId}`);

    // Apply effects
    if (action.effects) {
      this.applyEffects(action.effects as DialogueEffect[]);
    }

    this.evaluateRules(`action ${actionId}`);
    this.evaluateRules('always');
    this.checkSequenceAdvance();
    this.refreshScene();
    this.publishState();
  }

  /** Try to use a blocked exit — shows the blocked message. */
  tryBlockedExit(direction: string): void {
    if (this.state.status !== 'playing' || !this.state.currentLocation) return;

    const loc = this.locationMap.get(this.state.currentLocation.id);
    const exit = loc?.exits.find((e) => e.direction === direction);
    if (exit?.blocked_message) {
      this.pushNarrative('blocked', exit.blocked_message);
      this.emitEvent('condition', `Exit blocked: ${exit.blocked_message}`);
      this.publishState();
    }
  }

  destroy(): void {
    for (const unsub of this.unsubscribers) unsub();
    this.unsubscribers = [];
  }

  // ===== Entity helpers =====

  /** Get display name for an entity by id. */
  eName(id: string): string {
    const ent = this.entityState.get(id);
    if (!ent) return id;
    return (ent.properties.name as string) || (ent.properties.role as string) || id;
  }

  /** Get trait class for an entity. */
  private eTraitClass(id: string): 'interactable' | 'portable' | 'other' {
    const ent = this.entityState.get(id);
    if (!ent || !this.world?.types) return 'other';
    const td = this.world.types[ent.type];
    if (td?.traits?.includes('portable')) return 'portable';
    if (td?.traits?.includes('interactable')) return 'interactable';
    return 'other';
  }

  // ===== Private — Initialisation =====

  private initEntityState(): void {
    if (!this.world) return;
    this.entityState.clear();

    for (const ent of this.world.entities) {
      // Apply type defaults then entity-specific properties
      const defaults: Record<string, unknown> = {};
      const td = this.world.types?.[ent.type ?? ''];
      if (td?.properties) {
        for (const [k, p] of Object.entries(td.properties)) {
          if (p.default !== undefined) defaults[k] = p.default;
        }
      }

      this.entityState.set(ent.id, {
        type: ent.type ?? '',
        container: null,
        properties: { ...defaults, ...ent.properties },
      });
    }

    // Set containers from location.contains
    for (const loc of this.world.locations) {
      if (loc.contains) {
        for (const eid of loc.contains) {
          const ent = this.entityState.get(eid);
          if (ent) ent.container = loc.id;
        }
      }
    }
  }

  // ===== Private — Location =====

  private enterLocation(locationId: string): void {
    const loc = this.locationMap.get(locationId);
    if (!loc) return;

    // Clear narrative on location change
    this.state.narrative = [];
    this.narrativeCounter = 0;

    this.state.currentLocation = {
      id: loc.id,
      name: loc.name || fmtLoc(loc.id),
      description: loc.description || '',
      exits: loc.exits.map((e) => ({ direction: e.direction, target: e.target })),
    };

    this.state.visitedLocations = [...this.state.visitedLocations, loc.id];
    this.visitedLocations.add(loc.id);

    if (!this.coverage.locations.visited.includes(loc.id)) {
      this.coverage.locations = {
        ...this.coverage.locations,
        visited: [...this.coverage.locations.visited, loc.id],
      };
    }

    // Show location description as narration
    if (loc.description) {
      this.pushNarrative('narration', loc.description);
    }

    // Auto-start dialogue for entities with no-condition dialogues at this location
    this.autoStartDialogue(loc.id);

    this.refreshScene();
    this.publishState();
  }

  private autoStartDialogue(locationId: string): void {
    if (Object.keys(this.dialogue).length === 0) return;

    for (const [did, dlg] of Object.entries(this.dialogue)) {
      if (!dlg.prompt?.speaker) continue;
      const speaker = this.entityState.get(dlg.prompt.speaker);
      if (!speaker || speaker.container !== locationId) continue;
      // Only auto-enter dialogues with NO entry conditions
      if (dlg.conditions) continue;
      // Skip goto targets — navigated to from choices, not auto-triggered
      if (this.gotoTargets.has(did)) continue;
      // Skip dialogues with no choices — they are endpoints/transitions
      if (!dlg.choices || dlg.choices.length === 0) continue;
      // Skip exhausted sections
      if (this.exhaustedSections.has(did)) continue;
      if (this.dialogueState) continue;

      this.dialogueState = { dialogueId: did, subChoices: null };
      this.emitEvent('dialogue', `Auto-entered: ${did}`);

      // Show description before prompt
      if (dlg.description) {
        this.pushNarrative('narration', dlg.description);
      }

      this.pushNarrative('dlg_prompt', dlg.prompt.text, this.eName(dlg.prompt.speaker));

      // Check if exhausted immediately
      if (this.isSectionExhausted(did)) {
        this.handleExhaustedSection(did);
      }
      return;
    }
  }

  /** Walk the dialogue tree and collect all IDs referenced by goto fields. */
  private collectGotoTargets(): void {
    const collectFromChoices = (choices: DialogueChoice[]): void => {
      for (const ch of choices) {
        if (ch.goto && ch.goto !== 'end') this.gotoTargets.add(ch.goto);
        if (ch.choices) collectFromChoices(ch.choices);
      }
    };
    for (const dlg of Object.values(this.dialogue)) {
      if (dlg.choices) collectFromChoices(dlg.choices);
      if (dlg.on_exhausted?.goto) this.gotoTargets.add(dlg.on_exhausted.goto);
    }
  }

  // ===== Private — Scene refresh =====

  /** Rebuild all UI-facing state arrays from internal state. */
  private refreshScene(): void {
    if (!this.world || !this.state.currentLocation) return;

    const locationId = this.state.currentLocation.id;

    // Entities at location (excluding inventory and destroyed)
    const inventoryIds = new Set(this.state.inventory.map((i) => i.entityId));

    const entities: PlaybackEntity[] = [];
    for (const [id, ent] of this.entityState) {
      if (this.destroyedEntities.has(id)) continue;
      if (inventoryIds.has(id)) continue;
      if (ent.container !== locationId) continue;

      entities.push({
        id,
        name: this.eName(id),
        type: ent.type,
        properties: { ...ent.properties },
        container: locationId,
        traitClass: this.eTraitClass(id),
      });
    }
    this.state.entities = entities;

    // Exits
    const exits: PlaybackExitInfo[] = [];
    const loc = this.locationMap.get(locationId);
    if (loc) {
      for (const ex of loc.exits) {
        const blocked = ex.condition ? !this.evalCond(ex.condition) : false;
        exits.push({
          direction: ex.direction,
          targetId: ex.target,
          targetName: this.locationMap.get(ex.target)?.name || fmtLoc(ex.target),
          blocked,
          blockedMessage: ex.blocked_message,
        });
      }
    }
    this.state.availableExits = exits;

    // Dialogue state
    this.state.inDialogue = !!this.dialogueState;

    // Dialogue choices (filtered for one-shot exhaustion)
    if (this.dialogueState) {
      const availableChoices = this.getAvailableChoices();
      this.state.dialogueChoices = availableChoices.map((ch, i) => {
        const available = !ch.conditions || this.evalChoiceConds(ch.conditions);
        return {
          index: i,
          id: ch.id,
          label: ch.label,
          available,
          conditionText: ch.conditions && !available
            ? this.formatConditions(ch.conditions)
            : ch.conditions && available
              ? '\u2713'
              : undefined,
        };
      });
    } else {
      this.state.dialogueChoices = [];
    }

    // Interactions (talk-to buttons for interactable entities at this location)
    const interactions: PlaybackInteraction[] = [];
    if (!this.dialogueState) {
      const shown = new Set<string>();
      for (const [did, dlg] of Object.entries(this.dialogue)) {
        if (!dlg.prompt?.speaker) continue;
        const speaker = this.entityState.get(dlg.prompt.speaker);
        if (!speaker || speaker.container !== locationId) continue;
        if (this.destroyedEntities.has(dlg.prompt.speaker)) continue;
        if (shown.has(did)) continue;
        shown.add(did);

        const available = !dlg.conditions || this.evalConds(dlg.conditions);
        let conditionText: string | undefined;
        if (dlg.conditions && !available) {
          conditionText = this.formatConditions(dlg.conditions);
        }

        interactions.push({
          entityId: dlg.prompt.speaker,
          entityName: this.eName(dlg.prompt.speaker),
          dialogueId: did,
          available,
          conditionText,
        });
      }
    }
    this.state.interactions = interactions;

    // Pickups (portable items at this location)
    const pickups: PlaybackPickup[] = [];
    for (const [id, ent] of this.entityState) {
      if (this.destroyedEntities.has(id)) continue;
      if (ent.container !== locationId) continue;
      if (inventoryIds.has(id)) continue;
      if (this.eTraitClass(id) !== 'portable') continue;
      pickups.push({ entityId: id, entityName: this.eName(id) });
    }
    this.state.pickups = pickups;

    // Actions from the actions block
    this.state.availableActions = this.buildAvailableActions();

    // Sequence phase
    this.state.currentPhase = this.getActivePhase()?.id ?? null;

    // Legacy fields (for StateInspector, etc.)
    this.state.activeSection = this.dialogueState?.dialogueId ?? null;
    this.state.choices = this.state.dialogueChoices.map((dc) => ({
      id: dc.id,
      label: dc.label,
      sectionId: this.dialogueState?.dialogueId ?? '',
      available: dc.available,
    }));
  }

  // ===== Private — Dialogue =====

  /** Get choices for the current dialogue, filtering out used one-shot choices. */
  private getAvailableChoices(): DialogueChoice[] {
    if (!this.dialogueState) return [];
    const raw = this.dialogueState.subChoices
      ?? this.dialogue[this.dialogueState.dialogueId]?.choices
      ?? [];
    // Filter out used one-shot choices
    return raw.filter((ch) => ch.sticky || !this.usedChoices.has(ch.id));
  }

  /** Check if a section is exhausted (all choices used or unavailable). */
  private isSectionExhausted(sectionId: string): boolean {
    const dlg = this.dialogue[sectionId];
    if (!dlg?.choices || dlg.choices.length === 0) return true;

    return dlg.choices.every((ch) => {
      // One-shot and already used
      if (!ch.sticky && this.usedChoices.has(ch.id)) return true;
      // Has conditions that are all false
      if (ch.conditions && !this.evalChoiceConds(ch.conditions)) return true;
      return false;
    });
  }

  /** Update exhaustion tracking for a section. */
  private updateExhaustion(sectionId: string): void {
    if (this.isSectionExhausted(sectionId)) {
      this.exhaustedSections.add(sectionId);
      this.emitEvent('exhausted', `Section exhausted: ${sectionId}`);
    }
  }

  /** Handle an exhausted section — show on_exhausted or exit dialogue. */
  private handleExhaustedSection(sectionId: string): void {
    this.exhaustedSections.add(sectionId);
    const dlg = this.dialogue[sectionId];

    if (dlg?.on_exhausted) {
      const oe = dlg.on_exhausted;
      if (oe.speaker) {
        this.pushNarrative('dlg_prompt', oe.text, this.eName(oe.speaker));
      } else {
        this.pushNarrative('narration', oe.text);
      }

      if (oe.goto) {
        if (oe.goto === 'end') {
          this.endGame();
          return;
        }
        this.navigateToSection(oe.goto);
      } else {
        this.dialogueState = null;
      }
    } else {
      this.dialogueState = null;
    }

    this.refreshScene();
    this.publishState();
  }

  /** Navigate to a dialogue section by ID. */
  private navigateToSection(sectionId: string): void {
    const next = this.dialogue[sectionId];
    if (!next) {
      this.dialogueState = null;
      return;
    }

    this.dialogueState = { dialogueId: sectionId, subChoices: null };
    this.emitEvent('dialogue', `Goto: ${sectionId}`);

    // Track section coverage
    if (!this.coverage.sections.visited.includes(sectionId)) {
      this.coverage.sections = {
        ...this.coverage.sections,
        visited: [...this.coverage.sections.visited, sectionId],
      };
    }

    // Show description before prompt
    if (next.description) {
      this.pushNarrative('narration', next.description);
    }

    if (next.prompt) {
      this.pushNarrative('dlg_prompt', next.prompt.text, this.eName(next.prompt.speaker));
    }

    // Check if this section is exhausted
    if (this.isSectionExhausted(sectionId)) {
      this.handleExhaustedSection(sectionId);
    }
  }

  // ===== Private — Conditions =====

  private evalCond(condition: string): boolean {
    try {
      const c = condition.trim();

      // Exhaustion condition: "section_id.exhausted"
      const exhaustedMatch = c.match(/^(\S+)\.exhausted$/);
      if (exhaustedMatch) {
        return this.exhaustedSections.has(exhaustedMatch[1]);
      }

      // Container condition: "@entity in location" / "entity in player" / "entity in here"
      const inMatch = c.match(/^@?(\w+)\s+in\s+(\w+)$/);
      if (inMatch) {
        const [, eid, target] = inMatch;
        const ent = this.entityState.get(eid);
        if (!ent) return false;
        if (target === 'player') return ent.container === 'player';
        if (target === 'here') return ent.container === this.state.currentLocation?.id;
        return ent.container === target;
      }

      // Negated container condition: "@entity not in location"
      const notInMatch = c.match(/^@?(\w+)\s+not\s+in\s+(\w+)$/);
      if (notInMatch) {
        const [, eid, target] = notInMatch;
        const ent = this.entityState.get(eid);
        if (!ent) return true;
        if (target === 'player') return ent.container !== 'player';
        if (target === 'here') return ent.container !== this.state.currentLocation?.id;
        return ent.container !== target;
      }

      // Container property condition: "entity.container == target"
      // This is the compiler's lowered form for containment checks
      const containerMatch = c.match(/^@?(\w+)\.container\s*(==|!=)\s*(.+)$/);
      if (containerMatch) {
        const [, eid, op, rv] = containerMatch;
        const ent = this.entityState.get(eid);
        if (!ent) return false;
        let expected = rv.trim();
        if (expected === 'player.container') {
          expected = this.state.currentLocation?.id ?? '';
        }
        if (op === '==') return ent.container === expected;
        if (op === '!=') return ent.container !== expected;
        return false;
      }

      // Property condition: "entity.property op value"
      const m = c.match(/^@?(\w+)\.(\w+)\s*(==|!=|>=|<=|>|<)\s*(.+)$/);
      if (!m) {
        this.emitEvent('condition', `Unparsed: ${c}`);
        return false;
      }

      const [, eid, prop, op, rv] = m;
      const ent = this.entityState.get(eid);
      if (!ent) return false;

      const actual = ent.properties[prop];
      let expected: unknown = rv.trim();

      if (expected === 'true') expected = true;
      else if (expected === 'false') expected = false;
      else if (expected === 'player') expected = 'player';
      else if (expected === 'player.container') expected = this.state.currentLocation?.id;
      else if (typeof expected === 'string' && expected !== '' && !isNaN(Number(expected))) expected = Number(expected);
      else if (typeof expected === 'string') expected = expected.replace(/^["']|["']$/g, '');

      switch (op) {
        case '==': return actual == expected;
        case '!=': return actual != expected;
        case '>=': return (actual as number) >= (expected as number);
        case '<=': return (actual as number) <= (expected as number);
        case '>': return (actual as number) > (expected as number);
        case '<': return (actual as number) < (expected as number);
        default: return false;
      }
    } catch {
      return false;
    }
  }

  private evalConds(conditions: { any?: string[]; all?: string[] } | string[] | unknown): boolean {
    if (!conditions) return true;
    if (Array.isArray(conditions)) {
      return (conditions as string[]).every((c) => this.evalCond(c));
    }
    if (typeof conditions === 'object') {
      const obj = conditions as { any?: string[]; all?: string[] };
      if (obj.any) return obj.any.some((c) => this.evalCond(c));
      if (obj.all) return obj.all.every((c) => this.evalCond(c));
    }
    return true;
  }

  /** Evaluate choice conditions — same shapes as section conditions. */
  private evalChoiceConds(conditions: unknown): boolean {
    return this.evalConds(conditions);
  }

  /** Format conditions into a human-readable string. */
  private formatConditions(conditions: unknown): string {
    if (Array.isArray(conditions)) {
      return (conditions as string[]).join(', ');
    }
    if (typeof conditions === 'object' && conditions !== null) {
      const obj = conditions as { any?: string[]; all?: string[] };
      if (obj.any) return obj.any.join(' OR ');
      if (obj.all) return obj.all.join(' AND ');
    }
    return String(conditions);
  }

  // ===== Private — Effects =====

  private applyEffects(effects: DialogueEffect[]): void {
    for (const eff of effects) this.applyEffect(eff);
  }

  private applyEffect(effect: DialogueEffect): void {
    try {
      // Determine effect type by checking which key is present
      if ('set' in effect && 'to' in effect) {
        this.applySetEffect(effect.set, effect.to);
      } else if ('move' in effect && 'to' in effect) {
        this.applyMoveEffect(effect.move, effect.to);
      } else if ('destroy' in effect) {
        this.applyDestroyEffect(effect.destroy);
      } else if ('reveal' in effect) {
        this.applyRevealEffect(effect.reveal);
      } else {
        this.emitEvent('condition', `Unknown effect: ${JSON.stringify(effect)}`);
      }
    } catch {
      this.emitEvent('condition', `Effect error: ${JSON.stringify(effect)}`);
    }
  }

  private applySetEffect(target: string, value: string | number | boolean): void {
    const [eid, prop] = target.split('.');
    const ent = this.entityState.get(eid);
    if (!ent) {
      this.emitEvent('condition', `Effect target not found: ${eid}`);
      return;
    }

    let newValue: unknown = value;
    if (typeof value === 'string') {
      // Expression evaluation: e.g. "greta.trust + 1"
      const exprMatch = value.match(/^(\w+)\.(\w+)\s*([+\-])\s*(\d+)$/);
      if (exprMatch) {
        const base = (this.entityState.get(exprMatch[1])?.properties[exprMatch[2]] as number) ?? 0;
        newValue = exprMatch[3] === '+' ? base + Number(exprMatch[4]) : base - Number(exprMatch[4]);
      } else if (value === 'true') {
        newValue = true;
      } else if (value === 'false') {
        newValue = false;
      } else if (value !== '' && !isNaN(Number(value))) {
        newValue = Number(value);
      }
    }

    // Clamp integer values to type bounds
    const td = this.world?.types?.[ent.type];
    if (td?.properties?.[prop]) {
      const pd = td.properties[prop];
      if (pd.type === 'integer' && typeof newValue === 'number') {
        if (pd.min !== undefined) newValue = Math.max(pd.min, newValue as number);
        if (pd.max !== undefined) newValue = Math.min(pd.max, newValue as number);
      }
    }

    const oldValue = ent.properties[prop];
    ent.properties[prop] = newValue;
    this.emitEvent('effect', `${eid}.${prop}: ${JSON.stringify(oldValue)} -> ${JSON.stringify(newValue)}`);

    // Trigger state_change rules
    this.evaluateRules(`state_change ${prop}`);
  }

  private applyMoveEffect(entityId: string, destination: string): void {
    const ent = this.entityState.get(entityId);
    if (!ent) {
      this.emitEvent('condition', `Move target not found: ${entityId}`);
      return;
    }

    let resolvedDest = destination;
    if (destination === 'player.container') {
      resolvedDest = this.state.currentLocation?.id ?? '';
    }

    const oldContainer = ent.container;
    this.moveEntityTo(entityId, resolvedDest);
    this.emitEvent('effect', `Moved ${entityId}: ${oldContainer} -> ${resolvedDest}`);
  }

  private applyDestroyEffect(entityId: string): void {
    this.destroyedEntities.add(entityId);

    // Remove from inventory if present
    if (this.state.inventory.some((i) => i.entityId === entityId)) {
      this.state.inventory = this.state.inventory.filter((i) => i.entityId !== entityId);
    }

    this.emitEvent('effect', `Destroyed: ${entityId}`);
  }

  private applyRevealEffect(target: string): void {
    this.revealedProperties.add(target);
    this.emitEvent('effect', `Revealed: ${target}`);
  }

  /** Move an entity to a new container, updating inventory as needed. */
  private moveEntityTo(entityId: string, destination: string): void {
    const ent = this.entityState.get(entityId);
    if (!ent) return;

    const oldContainer = ent.container;
    ent.container = destination;

    // Update inventory
    if (destination === 'player') {
      if (!this.state.inventory.some((i) => i.entityId === entityId)) {
        this.state.inventory = [
          ...this.state.inventory,
          { entityId, name: this.eName(entityId), type: ent.type },
        ];
      }
    } else if (oldContainer === 'player') {
      // Removed from player's inventory
      this.state.inventory = this.state.inventory.filter((i) => i.entityId !== entityId);
    }
  }

  // ===== Private — Actions =====

  private buildAvailableActions(): PlaybackAction[] {
    if (!this.world?.actions || !this.state.currentLocation) return [];

    const locationId = this.state.currentLocation.id;
    const actions: PlaybackAction[] = [];

    for (const [id, action] of Object.entries(this.world.actions)) {
      // Check if the action's target is accessible
      let targetEntity: string | undefined;
      let relevant = false;

      if (action.target) {
        const ent = this.entityState.get(action.target);
        if (ent && !this.destroyedEntities.has(action.target)) {
          // Target must be at current location or in player inventory
          if (ent.container === locationId || ent.container === 'player') {
            relevant = true;
            targetEntity = action.target;
          }
        }
      } else if (action.target_type) {
        // Type-targeted: check if any entity of that type is at current location
        for (const [eid, ent] of this.entityState) {
          if (this.destroyedEntities.has(eid)) continue;
          if (ent.type === action.target_type && (ent.container === locationId || ent.container === 'player')) {
            relevant = true;
            targetEntity = eid;
            break;
          }
        }
      } else {
        // No target — always relevant (global action)
        relevant = true;
      }

      if (!relevant) continue;

      const available = !action.conditions || this.evalConds(action.conditions);
      let conditionText: string | undefined;
      if (action.conditions && !available) {
        conditionText = this.formatConditions(action.conditions);
      }

      actions.push({
        id,
        description: action.description,
        targetId: targetEntity,
        targetName: targetEntity ? this.eName(targetEntity) : undefined,
        available,
        conditionText,
      });
    }

    return actions;
  }

  // ===== Private — Sequences =====

  private activateSequence(sequenceId: string): void {
    if (!this.world?.sequences?.[sequenceId]) return;

    this.activeSequenceId = sequenceId;
    this.activePhaseIndex = 0;
    this.enterPhase();
  }

  private getActivePhase(): UrdSequencePhase | null {
    if (!this.activeSequenceId || !this.world?.sequences) return null;
    const seq = this.world.sequences[this.activeSequenceId];
    if (!seq) return null;
    return seq.phases[this.activePhaseIndex] ?? null;
  }

  private enterPhase(): void {
    const phase = this.getActivePhase();
    if (!phase) return;

    // Check phase condition
    if (phase.condition && !this.evalCond(phase.condition)) return;

    // Show prompt
    if (phase.prompt) {
      this.pushNarrative('system', phase.prompt);
    }

    // Apply phase effects
    if (phase.effects) {
      this.applyEffects(phase.effects as DialogueEffect[]);
    }

    this.emitEvent('effect', `Phase entered: ${phase.id}`);

    // If auto, advance immediately after effects
    if (phase.auto && phase.advance !== 'end') {
      this.advancePhase();
    }
  }

  private advancePhase(): void {
    if (!this.activeSequenceId || !this.world?.sequences) return;
    const seq = this.world.sequences[this.activeSequenceId];
    if (!seq) return;

    this.activePhaseIndex++;
    if (this.activePhaseIndex >= seq.phases.length) {
      // Sequence complete
      this.activeSequenceId = null;
      this.activePhaseIndex = 0;
      this.emitEvent('effect', 'Sequence completed');
      return;
    }

    this.enterPhase();
  }

  private checkSequenceAdvance(): void {
    const phase = this.getActivePhase();
    if (!phase) return;

    if (phase.advance === 'end') return;

    if (phase.advance === 'on_action') {
      // Advance on any action — called after action/choice
      this.advancePhase();
    } else if (phase.advance.startsWith('on_condition ')) {
      // Parse the condition expression (space-free encoded)
      const expr = phase.advance.slice('on_condition '.length);
      // Re-add spaces around operators for evalCond
      const normalised = expr.replace(/(==|!=|>=|<=|>|<)/g, ' $1 ');
      if (this.evalCond(normalised)) {
        this.advancePhase();
      }
    }
  }

  // ===== Private — Rules =====

  private evaluateRules(trigger: string): void {
    if (!this.world?.rules) return;

    for (const [ruleId, rule] of Object.entries(this.world.rules)) {
      if (this.firedRulesThisTurn.has(ruleId)) continue;

      // Check trigger match
      if (rule.trigger !== trigger && rule.trigger !== 'always') continue;

      // Check conditions
      if (rule.conditions && !rule.conditions.every((c) => this.evalCond(c))) continue;

      if (rule.select) {
        // Select-based rule: iterate over entities
        const { from, as: varName, where } = rule.select;
        for (const entityId of from) {
          const ent = this.entityState.get(entityId);
          if (!ent || this.destroyedEntities.has(entityId)) continue;

          // Check where conditions with variable substitution
          if (where) {
            const allMet = where.every((cond) => {
              const substituted = cond.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
              return this.evalCond(substituted);
            });
            if (!allMet) continue;
          }

          // Apply effects with variable substitution
          this.firedRulesThisTurn.add(ruleId);
          const substitutedEffects = rule.effects.map((eff) => {
            const effCopy = { ...eff };
            // Substitute variable name in effect targets
            if (effCopy.set) effCopy.set = effCopy.set.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
            if (effCopy.move) effCopy.move = effCopy.move.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
            if (effCopy.destroy) effCopy.destroy = effCopy.destroy.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
            if (effCopy.reveal) effCopy.reveal = effCopy.reveal.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
            if (typeof effCopy.to === 'string') {
              effCopy.to = effCopy.to.replace(new RegExp(`\\b${varName}\\b`, 'g'), entityId);
            }
            return effCopy;
          });
          this.applyEffects(substitutedEffects as DialogueEffect[]);
          this.emitEvent('effect', `Rule fired: ${ruleId} (target: ${entityId})`);
        }
      } else {
        // Simple rule: apply effects directly
        this.firedRulesThisTurn.add(ruleId);
        this.applyEffects(rule.effects as DialogueEffect[]);
        this.emitEvent('effect', `Rule fired: ${ruleId}`);
      }

      // Check if this rule firing should advance a sequence phase
      if (this.activeSequenceId) {
        const phase = this.getActivePhase();
        if (phase?.advance === 'on_rule' && phase.rule === ruleId) {
          this.advancePhase();
        }
      }
    }
  }

  // ===== Private — Game End =====

  private endGame(): void {
    this.state.gameOver = true;
    this.dialogueState = null;
    this.pushNarrative('system', 'The story has ended.');
    this.emitEvent('effect', 'Game ended');
    this.refreshScene();
    this.publishState();
  }

  // ===== Private — Helpers =====

  private countTotalExits(): number {
    if (!this.world) return 0;
    let count = 0;
    for (const loc of this.world.locations) {
      count += loc.exits.length;
    }
    return count;
  }

  private pushNarrative(kind: NarrativeLineKind, text: string, speaker?: string): void {
    const line: NarrativeLine = {
      id: this.narrativeCounter++,
      kind,
      text,
      speaker,
      timestamp: Date.now(),
    };
    this.state.narrative = [...this.state.narrative, line];
  }

  private emitEvent(type: PlaybackEventType, summary: string, details?: Record<string, unknown>): void {
    const event: PlaybackEvent = {
      id: this.eventCounter++,
      type,
      timestamp: Date.now(),
      summary,
      details,
    };
    this.events = [...this.events, event];
    bus.publish('playback.event', { event });
  }

  private publishState(): void {
    bus.publish('playback.state.changed', { state: this.state });
    bus.publish('coverage.overlay.updated', { coverage: this.coverage });
  }
}

export const playbackService = new PlaybackServiceImpl();
