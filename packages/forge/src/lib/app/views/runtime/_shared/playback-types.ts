/**
 * Playback types — shared data model for all runtime views.
 *
 * These types define the contract between the PlaybackService and
 * the runtime view components (PlayPanel, StateInspector, EventLog,
 * BreadcrumbTrail, CoverageOverlay).
 */

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
}

export interface PlaybackChoice {
  id: string;
  label: string;
  sectionId: string;
  available: boolean;
}

export interface PlaybackState {
  status: PlaybackStatus;
  currentLocation: PlaybackLocation | null;
  entities: PlaybackEntity[];
  availableExits: Array<{ direction: string; targetId: string; targetName: string }>;
  activeSection: string | null;
  choices: PlaybackChoice[];
  visitedLocations: string[];
  turnCount: number;
}

// ===== Playback events =====

export type PlaybackEventType =
  | 'move'
  | 'narration'
  | 'dialogue'
  | 'set'
  | 'section_enter'
  | 'choice_made'
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
