/**
 * PlaybackService — mock runtime that simulates basic world interaction
 * using compiled urdJson + factSet data.
 *
 * When @urd/wyrd-core is built, this implementation will be replaced
 * while keeping the same public interface.
 */

import { bus } from '$lib/framework/bus/MessageBus';
import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
import type { UrdWorld, UrdLocation, FactSet } from '$lib/app/compiler/types';
import {
  type PlaybackState,
  type PlaybackEvent,
  type PlaybackEventType,
  type PlaybackEntity,
  type PlaybackChoice,
  type CoverageData,
  createInitialPlaybackState,
  createInitialCoverage,
} from './playback-types';

class PlaybackServiceImpl {
  state: PlaybackState = $state(createInitialPlaybackState());
  events: PlaybackEvent[] = $state([]);
  coverage: CoverageData = $state(createInitialCoverage());

  private world: UrdWorld | null = null;
  private factSet: FactSet | null = null;
  private locationMap: Map<string, UrdLocation> = new Map();
  private consumedChoices: Set<string> = new Set();
  private visitedSections: Set<string> = new Set();
  private traversedExits: Set<string> = new Set();
  private eventCounter = 0;
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
    this.consumedChoices.clear();
    this.visitedSections.clear();
    this.traversedExits.clear();
    this.eventCounter = 0;
    this.events = [];

    for (const loc of urdJson.locations) {
      this.locationMap.set(loc.id, loc);
    }

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

    // Start at world.start or first location
    const startId = urdJson.world?.start ?? urdJson.locations[0]?.id;
    if (startId) {
      this.enterLocation(startId);
    }

    this.state.status = 'playing';
    this.emitEvent('world_loaded', `World loaded: ${urdJson.world?.name ?? 'Untitled'}`);
    this.publishState();
  }

  reset(): void {
    if (!this.world) return;

    this.consumedChoices.clear();
    this.visitedSections.clear();
    this.traversedExits.clear();
    this.eventCounter = 0;
    this.events = [];

    // Reset coverage visited sets but keep totals
    this.coverage = {
      locations: { visited: [], total: this.coverage.locations.total },
      sections: { visited: [], total: this.coverage.sections.total },
      choices: { visited: [], total: this.coverage.choices.total },
      exits: { visited: [], total: this.coverage.exits.total },
    };

    const startId = this.world.world?.start ?? this.world.locations[0]?.id;
    this.state = createInitialPlaybackState();

    if (startId) {
      this.enterLocation(startId);
    }

    this.state.status = 'playing';
    this.emitEvent('world_reset', 'World reset to initial state');
    this.publishState();
  }

  move(direction: string): void {
    if (this.state.status !== 'playing' || !this.state.currentLocation) return;

    const exit = this.state.currentLocation.exits.find((e) => e.direction === direction);
    if (!exit) return;

    const exitKey = `${this.state.currentLocation.id}/${direction}`;
    if (!this.traversedExits.has(exitKey)) {
      this.traversedExits.add(exitKey);
      this.coverage.exits = {
        ...this.coverage.exits,
        visited: [...this.coverage.exits.visited, exitKey],
      };
    }

    const fromName = this.state.currentLocation.name;
    this.enterLocation(exit.target);
    this.state.turnCount++;

    this.emitEvent('move', `Moved ${direction} from ${fromName} to ${this.state.currentLocation!.name}`, {
      direction,
      from: fromName,
      to: this.state.currentLocation!.id,
    });
    this.publishState();
  }

  enterSection(sectionId: string): void {
    if (this.state.status !== 'playing') return;

    this.state.activeSection = sectionId;

    if (!this.visitedSections.has(sectionId)) {
      this.visitedSections.add(sectionId);
      this.coverage.sections = {
        ...this.coverage.sections,
        visited: [...this.coverage.sections.visited, sectionId],
      };
    }

    this.refreshChoices();
    this.emitEvent('section_enter', `Entered section: ${sectionId}`, { sectionId });
    this.publishState();
  }

  choose(choiceId: string): void {
    if (this.state.status !== 'playing') return;

    const choice = this.state.choices.find((c) => c.id === choiceId);
    if (!choice || !choice.available) return;

    // Find the FactSet choice to determine if sticky or one-shot
    const factChoice = this.factSet?.choices.find((c) => c.choice_id === choiceId);
    if (factChoice && !factChoice.sticky) {
      this.consumedChoices.add(choiceId);
    }

    if (!this.coverage.choices.visited.includes(choiceId)) {
      this.coverage.choices = {
        ...this.coverage.choices,
        visited: [...this.coverage.choices.visited, choiceId],
      };
    }

    this.refreshChoices();
    this.state.turnCount++;

    this.emitEvent('choice_made', `Chose: ${choice.label}`, {
      choiceId,
      label: choice.label,
      sectionId: choice.sectionId,
    });

    // Follow any jumps from this choice
    if (factChoice) {
      for (const ji of factChoice.jump_indices) {
        const jump = this.factSet?.jumps[ji];
        if (jump) {
          if (jump.target.kind === 'section' && jump.target.id) {
            this.enterSection(jump.target.id);
            return;
          } else if (jump.target.kind === 'end') {
            this.state.activeSection = null;
            this.state.choices = [];
            this.emitEvent('exhausted', 'Dialogue ended');
            this.publishState();
            return;
          }
        }
      }
    }

    this.publishState();
  }

  destroy(): void {
    for (const unsub of this.unsubscribers) unsub();
    this.unsubscribers = [];
  }

  // ===== Private =====

  private enterLocation(locationId: string): void {
    const loc = this.locationMap.get(locationId);
    if (!loc) return;

    this.state.currentLocation = {
      id: loc.id,
      name: loc.name || loc.id,
      description: loc.description || '',
      exits: loc.exits.map((e) => ({ direction: e.direction, target: e.target })),
    };

    this.state.availableExits = loc.exits.map((e) => ({
      direction: e.direction,
      targetId: e.target,
      targetName: this.locationMap.get(e.target)?.name || e.target,
    }));

    // Track visited locations (avoid duplicates in the trail for coverage, but allow in breadcrumb)
    this.state.visitedLocations = [...this.state.visitedLocations, loc.id];

    if (!this.coverage.locations.visited.includes(loc.id)) {
      this.coverage.locations = {
        ...this.coverage.locations,
        visited: [...this.coverage.locations.visited, loc.id],
      };
    }

    // Collect entities at this location
    this.refreshEntities(loc.id);

    // Check for dialogue sections available at this location
    this.refreshSectionsForLocation(loc.id);
  }

  private refreshEntities(locationId: string): void {
    if (!this.world) return;

    const entities: PlaybackEntity[] = [];
    const location = this.locationMap.get(locationId);
    const containedIds = new Set(location?.contains ?? []);

    for (const entity of this.world.entities) {
      // Show entities contained by this location, or all if no containment
      if (containedIds.size > 0 && !containedIds.has(entity.id)) continue;

      entities.push({
        id: entity.id,
        name: entity.name || entity.id,
        type: entity.type,
        properties: { ...entity.properties },
        container: locationId,
      });
    }

    this.state.entities = entities;
  }

  private refreshSectionsForLocation(locationId: string): void {
    if (!this.factSet) {
      this.state.activeSection = null;
      this.state.choices = [];
      return;
    }

    // Find sections that have choices (available dialogue)
    const sectionIds = new Set<string>();
    for (const choice of this.factSet.choices) {
      sectionIds.add(choice.section);
    }

    // If there are sections and none is active, auto-enter the first one
    if (sectionIds.size > 0 && !this.state.activeSection) {
      const firstSection = [...sectionIds][0];
      this.state.activeSection = firstSection;

      if (!this.visitedSections.has(firstSection)) {
        this.visitedSections.add(firstSection);
        this.coverage.sections = {
          ...this.coverage.sections,
          visited: [...this.coverage.sections.visited, firstSection],
        };
      }
    }

    this.refreshChoices();
  }

  private refreshChoices(): void {
    if (!this.factSet || !this.state.activeSection) {
      this.state.choices = [];
      return;
    }

    const choices: PlaybackChoice[] = [];
    for (const c of this.factSet.choices) {
      if (c.section !== this.state.activeSection) continue;

      const available = !this.consumedChoices.has(c.choice_id);
      choices.push({
        id: c.choice_id,
        label: c.label,
        sectionId: c.section,
        available,
      });
    }

    this.state.choices = choices;

    // Check for exhaustion — all choices consumed
    if (choices.length > 0 && choices.every((c) => !c.available)) {
      this.emitEvent('exhausted', `Section exhausted: ${this.state.activeSection}`);
      this.state.activeSection = null;
      this.state.choices = [];
    }
  }

  private countTotalExits(): number {
    if (!this.world) return 0;
    let count = 0;
    for (const loc of this.world.locations) {
      count += loc.exits.length;
    }
    return count;
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
