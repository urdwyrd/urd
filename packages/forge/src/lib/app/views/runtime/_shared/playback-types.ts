/**
 * Playback types — shared data model for all runtime views.
 *
 * These types define the contract between the PlaybackService and
 * the runtime view components (PlayPanel, StateInspector, EventLog,
 * BreadcrumbTrail, CoverageOverlay).
 */

// ===== Dialogue tree types (consumed by the runtime from urdJson.dialogue) =====

export interface DialoguePrompt {
  speaker: string;
  text: string;
}

export interface EffectSet { set: string; to: string | number | boolean; }
export interface EffectMove { move: string; to: string; }
export interface EffectDestroy { destroy: string; }
export interface EffectReveal { reveal: string; }
export type DialogueEffect = EffectSet | EffectMove | EffectDestroy | EffectReveal;

export interface DialogueChoice {
  id: string;
  label: string;
  sticky?: boolean;
  conditions?: string[];
  response?: DialoguePrompt;
  effects?: DialogueEffect[];
  goto?: string;
  choices?: DialogueChoice[];
}

export interface DialogueOnExhausted {
  speaker?: string;
  text: string;
  goto?: string;
}

export interface DialogueNode {
  id: string;
  prompt?: DialoguePrompt;
  description?: string;
  conditions?: { any?: string[]; all?: string[] } | string[];
  choices: DialogueChoice[];
  on_exhausted?: DialogueOnExhausted;
}

// ===== Narrative line (IF-style narrative entries) =====

export type NarrativeLineKind =
  | 'narration'       // Italic narrator prose (location descriptions, scene-setting)
  | 'dlg_prompt'      // NPC dialogue prompt (speaker initiating conversation)
  | 'dlg_response'    // NPC dialogue response (speaker replying to player choice)
  | 'player_action'   // Player action (chose something, picked up, moved)
  | 'system'          // System messages (world loaded, phase changes)
  | 'blocked';        // Blocked action (exit locked, condition not met)

export interface NarrativeLine {
  id: number;
  kind: NarrativeLineKind;
  text: string;
  speaker?: string;
  timestamp: number;
}

// ===== Runtime entity state (mutable copy maintained by PlaybackService) =====

export interface RuntimeEntity {
  type: string;
  container: string | null;
  properties: Record<string, unknown>;
}

// ===== Playback state =====

export type PlaybackStatus = 'idle' | 'playing' | 'error';

export interface PlaybackLocation {
  id: string;
  name: string;
  description: string;
  exits: Array<{ direction: string; target: string }>;
}

export interface PlaybackEntity {
  id: string;
  name: string;
  type?: string;
  properties: Record<string, unknown>;
  container?: string;
  /** Entity trait classification for UI display. */
  traitClass: 'interactable' | 'portable' | 'other';
}

export interface PlaybackChoice {
  id: string;
  label: string;
  sectionId: string;
  available: boolean;
}

export interface PlaybackInventoryItem {
  entityId: string;
  name: string;
  type?: string;
}

export interface PlaybackExitInfo {
  direction: string;
  targetId: string;
  targetName: string;
  blocked: boolean;
  blockedMessage?: string;
}

export interface PlaybackInteraction {
  entityId: string;
  entityName: string;
  dialogueId: string;
  available: boolean;
  conditionText?: string;
}

export interface PlaybackPickup {
  entityId: string;
  entityName: string;
}

export interface PlaybackAction {
  id: string;
  description?: string;
  targetId?: string;
  targetName?: string;
  available: boolean;
  conditionText?: string;
}

export interface PlaybackState {
  status: PlaybackStatus;
  currentLocation: PlaybackLocation | null;
  entities: PlaybackEntity[];
  availableExits: PlaybackExitInfo[];
  activeSection: string | null;
  choices: PlaybackChoice[];
  visitedLocations: string[];
  turnCount: number;
  narrative: NarrativeLine[];
  inventory: PlaybackInventoryItem[];
  inDialogue: boolean;
  /** Dialogue choices with condition info (richer than PlaybackChoice). */
  dialogueChoices: DialogueChoiceView[];
  /** Available NPC interactions at current location. */
  interactions: PlaybackInteraction[];
  /** Portable items available for pickup at current location. */
  pickups: PlaybackPickup[];
  /** Available actions from the actions block. */
  availableActions: PlaybackAction[];
  /** Whether the game has ended (-> end reached). */
  gameOver: boolean;
  /** Active sequence phase ID, if a sequence is running. */
  currentPhase: string | null;
}

/** Dialogue choice as presented to the UI — includes availability and condition text. */
export interface DialogueChoiceView {
  index: number;
  id: string;
  label: string;
  available: boolean;
  conditionText?: string;
}

// ===== Playback events =====

export type PlaybackEventType =
  | 'move'
  | 'narration'
  | 'dialogue'
  | 'set'
  | 'effect'
  | 'condition'
  | 'section_enter'
  | 'choice_made'
  | 'interact'
  | 'exhausted'
  | 'world_loaded'
  | 'world_reset';

export interface PlaybackEvent {
  id: number;
  type: PlaybackEventType;
  timestamp: number;
  summary: string;
  details?: Record<string, unknown>;
}

// ===== Coverage =====

export interface CoverageCategory {
  visited: string[];
  total: number;
}

export interface CoverageData {
  locations: CoverageCategory;
  sections: CoverageCategory;
  choices: CoverageCategory;
  exits: CoverageCategory;
}

// ===== Initial state factory =====

export function createInitialPlaybackState(): PlaybackState {
  return {
    status: 'idle',
    currentLocation: null,
    entities: [],
    availableExits: [],
    activeSection: null,
    choices: [],
    visitedLocations: [],
    turnCount: 0,
    narrative: [],
    inventory: [],
    inDialogue: false,
    dialogueChoices: [],
    interactions: [],
    pickups: [],
    availableActions: [],
    gameOver: false,
    currentPhase: null,
  };
}

export function createInitialCoverage(): CoverageData {
  return {
    locations: { visited: [], total: 0 },
    sections: { visited: [], total: 0 },
    choices: { visited: [], total: 0 },
    exits: { visited: [], total: 0 },
  };
}
