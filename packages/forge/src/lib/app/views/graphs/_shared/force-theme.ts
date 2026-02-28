/**
 * Theme bridge — reads CSS custom properties into typed JS values
 * for Canvas 2D and Three.js renderers that cannot use CSS directly.
 */

export interface GraphTheme {
  bgPrimary: string;
  textPrimary: string;
  textMuted: string;
  accentPrimary: string;
  accentSecondary: string;
  borderZone: string;
  nodeDefault: string;
  edgeDefault: string;
  nodeSelected: string;
  nodeStart: string;
  nodeUnreachable: string;
  nodeIsolated: string;
  edgeConditional: string;
  edgeChoice: string;
  edgeTerminal: string;
}

/** Read graph theme colours from CSS custom properties. */
export function readGraphTheme(): GraphTheme {
  const s = getComputedStyle(document.documentElement);
  const get = (prop: string): string => s.getPropertyValue(prop).trim() || '#888';
  return {
    bgPrimary: get('--forge-bg-primary'),
    textPrimary: get('--forge-text-primary'),
    textMuted: get('--forge-text-muted'),
    accentPrimary: get('--forge-accent-primary'),
    accentSecondary: get('--forge-accent-secondary'),
    borderZone: get('--forge-border-zone'),
    nodeDefault: get('--forge-graph-node-default'),
    edgeDefault: get('--forge-graph-edge-default'),
    nodeSelected: get('--forge-graph-node-selected'),
    nodeStart: get('--forge-graph-node-start'),
    nodeUnreachable: get('--forge-graph-node-unreachable'),
    nodeIsolated: get('--forge-graph-node-isolated'),
    edgeConditional: get('--forge-graph-edge-conditional'),
    edgeChoice: get('--forge-graph-edge-choice'),
    edgeTerminal: get('--forge-graph-edge-terminal'),
  };
}

/** Convert CSS hex colour to numeric 0xRRGGBB for Three.js. */
export function hexToNumber(hex: string): number {
  const clean = hex.replace('#', '');
  return parseInt(clean, 16);
}

/** Convert CSS hex colour to [r, g, b] floats (0..1) for Three.js. */
export function hexToRgb(hex: string): [number, number, number] {
  const n = hexToNumber(hex);
  return [(n >> 16) / 255, ((n >> 8) & 0xff) / 255, (n & 0xff) / 255];
}
