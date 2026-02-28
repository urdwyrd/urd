<script lang="ts">
  /**
   * WorldCanvas — rich Canvas 2D renderer for spatial Urd world visualisation.
   *
   * Renders locations as large cards with entity icons inside, curved exit
   * arrows with condition labels, and a runtime state overlay showing the
   * player's current position, visited locations, and property changes.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import type {
    WorldData,
    WorldLocation,
    WorldEntity,
    WorldExit,
    WorldScene,
    WorldRuntimeOverlay,
    EntityIconKind,
  } from './world-types';
  import { readGraphTheme, type GraphTheme } from './force-theme';

  interface Props {
    data: WorldData;
    runtimeOverlay?: WorldRuntimeOverlay | null;
    onLocationClick?: (locationId: string) => void;
    onEntityClick?: (entityId: string) => void;
    onLocationDblClick?: (locationId: string) => void;
    onEntityDblClick?: (entityId: string) => void;
    onExitClick?: (fromId: string, direction: string) => void;
    onNodeMoved?: (locationId: string, x: number, y: number) => void;
    emptyMessage?: string;
  }

  let {
    data,
    runtimeOverlay,
    onLocationClick,
    onEntityClick,
    onLocationDblClick,
    onEntityDblClick,
    onExitClick,
    onNodeMoved,
    emptyMessage = 'No world data to display',
  }: Props = $props();

  let containerEl: HTMLDivElement | undefined = $state(undefined);
  let canvasEl: HTMLCanvasElement | undefined = $state(undefined);
  let ctx: CanvasRenderingContext2D | null = null;
  let theme: GraphTheme | null = null;

  // Viewport
  let camX = $state(0);
  let camY = $state(0);
  let zoom = $state(1);

  // Interaction
  let isPanning = false;
  let isDraggingNode = false;
  let draggedLocation: WorldLocation | null = null;
  let dragOffsetX = 0;
  let dragOffsetY = 0;
  let panStartX = 0;
  let panStartY = 0;
  let panOriginCamX = 0;
  let panOriginCamY = 0;
  let pointerDownWorldX = 0;
  let pointerDownWorldY = 0;

  // Double-click detection
  let lastClickTime = 0;
  let lastClickTarget: string | null = null;
  const DBLCLICK_MS = 350;

  let hoveredLocationId: string | null = $state(null);
  let hoveredEntityId: string | null = $state(null);
  let tooltipEntity: WorldEntity | null = $state(null);
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  // Layout constants
  const LOC_PADDING = 16;
  const LOC_HEADER_H = 40;
  const LOC_DESC_H = 24;
  const ENTITY_ROW_H = 28;
  const ENTITY_ICON_SIZE = 18;
  const ENTITY_COL_W = 150;
  const LOC_MIN_W = 220;
  const LOC_BORDER_R = 8;
  const EXIT_ARROW_SIZE = 8;
  const SCENE_RADIUS = 24;
  const CHOICE_H = 20;

  // Filter state
  let showEntities = $state(true);
  let showExits = $state(true);
  let showScenes = $state(true);

  let rafId = 0;
  let canvasW = 0;
  let canvasH = 0;
  let dpr = 1;

  // Pre-built location position map for exit routing
  let locationCenters: Map<string, { x: number; y: number; w: number; h: number }> = new Map();

  const resizeObserver = new ResizeObserver(() => resizeCanvas());

  let initialized = false;
  let lastDataLocationCount = -1;

  function initCanvas(): void {
    if (initialized || !canvasEl || !containerEl) return;
    ctx = canvasEl.getContext('2d');
    theme = readGraphTheme();
    dpr = window.devicePixelRatio || 1;
    resizeCanvas();
    resizeObserver.observe(containerEl);
    centreOnWorld();
    lastDataLocationCount = data.locations.length;
    startLoop();
    initialized = true;
  }

  let unsubTheme: (() => void) | null = null;

  onMount(() => {
    initCanvas();
    unsubTheme = bus.subscribe('theme.changed', () => {
      theme = readGraphTheme();
    });
  });

  onDestroy(() => {
    cancelAnimationFrame(rafId);
    resizeObserver.disconnect();
    unsubTheme?.();
  });

  $effect(() => {
    // Init canvas when elements appear
    if (canvasEl && containerEl) {
      initCanvas();
    }
    // Rebuild location centres whenever data changes (needed for exit routing)
    if (data) {
      buildLocationCenters();
      // Only auto-centre when data structurally changes (new compilation),
      // not on every re-render — otherwise it fights the user's zoom/pan.
      if (data.locations.length !== lastDataLocationCount) {
        lastDataLocationCount = data.locations.length;
        centreOnWorld();
      }
    }
  });

  function buildLocationCenters(): void {
    locationCenters.clear();
    for (const loc of data.locations) {
      locationCenters.set(loc.id, {
        x: loc.x + loc.width / 2,
        y: loc.y + loc.height / 2,
        w: loc.width,
        h: loc.height,
      });
    }
  }

  function resizeCanvas(): void {
    if (!containerEl || !canvasEl) return;
    const rect = containerEl.getBoundingClientRect();
    canvasW = rect.width;
    canvasH = rect.height;
    canvasEl.width = canvasW * dpr;
    canvasEl.height = canvasH * dpr;
    canvasEl.style.width = `${canvasW}px`;
    canvasEl.style.height = `${canvasH}px`;
  }

  function centreOnWorld(): void {
    if (data.locations.length === 0) return;
    // Find bounding box
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const loc of data.locations) {
      minX = Math.min(minX, loc.x);
      minY = Math.min(minY, loc.y);
      maxX = Math.max(maxX, loc.x + loc.width);
      maxY = Math.max(maxY, loc.y + loc.height);
    }
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    const worldW = maxX - minX + 100;
    const worldH = maxY - minY + 100;
    // Fit zoom
    if (canvasW > 0 && canvasH > 0) {
      zoom = Math.min(1.2, canvasW / worldW, canvasH / worldH);
      zoom = Math.max(0.15, zoom);
    }
    camX = -cx;
    camY = -cy;
  }

  function startLoop(): void {
    function frame() {
      draw();
      rafId = requestAnimationFrame(frame);
    }
    rafId = requestAnimationFrame(frame);
  }

  // ===== Drawing =====

  function draw(): void {
    if (!ctx || !theme) return;
    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, canvasW, canvasH);

    // Background
    ctx.fillStyle = theme.bgPrimary;
    ctx.fillRect(0, 0, canvasW, canvasH);

    // Draw subtle grid
    drawGrid();

    // Transform to world space
    ctx.save();
    ctx.translate(canvasW / 2, canvasH / 2);
    ctx.scale(zoom, zoom);
    ctx.translate(camX, camY);

    // Draw exits (under locations)
    if (showExits) {
      for (const loc of data.locations) {
        for (const exit of loc.exits) {
          drawExit(loc, exit);
        }
      }
    }

    // Draw locations
    for (const loc of data.locations) {
      drawLocation(loc);
    }

    // Draw scenes
    if (showScenes) {
      for (const scene of data.scenes) {
        drawScene(scene);
      }
    }

    ctx.restore();
    ctx.restore();
  }

  function drawGrid(): void {
    if (!ctx || !theme) return;
    const gridSize = 60 * zoom;
    if (gridSize < 10) return; // Too zoomed out

    ctx.strokeStyle = theme.borderZone;
    ctx.globalAlpha = 0.08;
    ctx.lineWidth = 1;

    const offsetX = ((camX * zoom + canvasW / 2) % gridSize + gridSize) % gridSize;
    const offsetY = ((camY * zoom + canvasH / 2) % gridSize + gridSize) % gridSize;

    for (let x = offsetX; x < canvasW; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvasH);
      ctx.stroke();
    }
    for (let y = offsetY; y < canvasH; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvasW, y);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
  }

  function drawLocation(loc: WorldLocation): void {
    if (!ctx || !theme) return;
    const isHovered = hoveredLocationId === loc.id;
    const isCurrent = runtimeOverlay?.currentLocationId === loc.id;
    const isVisited = runtimeOverlay?.visitedLocationIds.has(loc.id) ?? false;

    // Card shadow
    ctx.save();
    ctx.shadowColor = 'rgba(0,0,0,0.25)';
    ctx.shadowBlur = isHovered ? 16 : 8;
    ctx.shadowOffsetY = 2;

    // Card background
    ctx.fillStyle = theme.bgPrimary;
    roundRect(ctx, loc.x, loc.y, loc.width, loc.height, LOC_BORDER_R);
    ctx.fill();
    ctx.restore();

    // Border
    ctx.strokeStyle = isCurrent
      ? (theme.nodeStart)
      : isVisited
        ? (theme.accentPrimary + '80')
        : isHovered
          ? theme.accentPrimary
          : theme.borderZone;
    ctx.lineWidth = isCurrent ? 3 : isHovered ? 2 : 1.5;
    roundRect(ctx, loc.x, loc.y, loc.width, loc.height, LOC_BORDER_R);
    ctx.stroke();

    // Current location glow
    if (isCurrent) {
      ctx.save();
      ctx.strokeStyle = theme.nodeStart;
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.3;
      roundRect(ctx, loc.x - 4, loc.y - 4, loc.width + 8, loc.height + 8, LOC_BORDER_R + 2);
      ctx.stroke();
      ctx.restore();
    }

    // Header bar
    ctx.fillStyle = isCurrent
      ? (theme.nodeStart + '20')
      : isVisited
        ? (theme.accentPrimary + '10')
        : (theme.borderZone + '20');
    ctx.beginPath();
    ctx.moveTo(loc.x + LOC_BORDER_R, loc.y);
    ctx.lineTo(loc.x + loc.width - LOC_BORDER_R, loc.y);
    ctx.arcTo(loc.x + loc.width, loc.y, loc.x + loc.width, loc.y + LOC_BORDER_R, LOC_BORDER_R);
    ctx.lineTo(loc.x + loc.width, loc.y + LOC_HEADER_H);
    ctx.lineTo(loc.x, loc.y + LOC_HEADER_H);
    ctx.lineTo(loc.x, loc.y + LOC_BORDER_R);
    ctx.arcTo(loc.x, loc.y, loc.x + LOC_BORDER_R, loc.y, LOC_BORDER_R);
    ctx.closePath();
    ctx.fill();

    // Location name
    ctx.fillStyle = theme.textPrimary;
    ctx.font = 'bold 13px Outfit, system-ui, sans-serif';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    const nameX = loc.x + LOC_PADDING;
    const nameY = loc.y + LOC_HEADER_H / 2;
    ctx.fillText(truncate(loc.name, 24), nameX, nameY);

    // Start badge
    if (loc.isStart) {
      const badgeText = 'START';
      ctx.font = 'bold 9px Outfit, system-ui, sans-serif';
      const badgeW = ctx.measureText(badgeText).width + 10;
      const badgeX = loc.x + loc.width - LOC_PADDING - badgeW;
      const badgeY = loc.y + LOC_HEADER_H / 2 - 8;
      ctx.fillStyle = theme.nodeStart;
      roundRect(ctx, badgeX, badgeY, badgeW, 16, 4);
      ctx.fill();
      ctx.fillStyle = theme.bgPrimary;
      ctx.textAlign = 'center';
      ctx.fillText(badgeText, badgeX + badgeW / 2, badgeY + 9);
    }

    // Description (if zoomed in enough)
    if (zoom > 0.4 && loc.description) {
      ctx.fillStyle = theme.textMuted;
      ctx.font = '11px "Source Serif 4", Georgia, serif';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'top';
      const descY = loc.y + LOC_HEADER_H + 6;
      const maxDescW = loc.width - LOC_PADDING * 2;
      ctx.fillText(truncate(loc.description, Math.floor(maxDescW / 5.5)), nameX, descY);
    }

    // Entities
    if (showEntities && loc.entities.length > 0 && zoom > 0.3) {
      const entityStartY = loc.y + LOC_HEADER_H + LOC_DESC_H + 8;
      const cols = Math.max(1, Math.floor((loc.width - LOC_PADDING) / ENTITY_COL_W));

      for (let i = 0; i < loc.entities.length; i++) {
        const entity = loc.entities[i];
        const col = i % cols;
        const row = Math.floor(i / cols);
        const ex = loc.x + LOC_PADDING + col * ENTITY_COL_W;
        const ey = entityStartY + row * ENTITY_ROW_H;

        const isEntityHovered = hoveredEntityId === entity.id;
        const hasRuntimeState = runtimeOverlay?.entityStates.has(entity.id) ?? false;

        // Entity icon
        drawEntityIcon(ex, ey + ENTITY_ROW_H / 2, entity.entityKind, isEntityHovered, hasRuntimeState);

        // Entity name
        ctx.fillStyle = isEntityHovered ? theme!.accentPrimary : theme!.textPrimary;
        ctx.font = `${isEntityHovered ? 'bold ' : ''}11px Outfit, system-ui, sans-serif`;
        ctx.textAlign = 'left';
        ctx.textBaseline = 'middle';
        ctx.fillText(truncate(entity.name, 16), ex + ENTITY_ICON_SIZE + 6, ey + ENTITY_ROW_H / 2);

        // Property count badge
        const propCount = Object.keys(entity.properties).length;
        if (propCount > 0 && zoom > 0.6) {
          const badgeText = `${propCount}`;
          ctx.font = '9px "JetBrains Mono", monospace';
          ctx.fillStyle = theme!.textMuted;
          const bx = ex + ENTITY_COL_W - 28;
          ctx.fillText(badgeText, bx, ey + ENTITY_ROW_H / 2);
        }
      }
    }

    // Exit direction badges along card border
    if (showExits && zoom > 0.5) {
      for (const exit of loc.exits) {
        drawExitBadge(loc, exit);
      }
    }
  }

  function drawEntityIcon(cx: number, cy: number, kind: EntityIconKind, hovered: boolean, runtimeActive: boolean): void {
    if (!ctx || !theme) return;
    const r = ENTITY_ICON_SIZE / 2;

    ctx.save();
    if (hovered) {
      ctx.shadowColor = theme.accentPrimary;
      ctx.shadowBlur = 6;
    }

    switch (kind) {
      case 'character': {
        // Circle
        ctx.beginPath();
        ctx.arc(cx + r, cy, r - 1, 0, Math.PI * 2);
        ctx.fillStyle = runtimeActive ? theme.nodeStart + '40' : '#4a90d9' + '30';
        ctx.fill();
        ctx.strokeStyle = runtimeActive ? theme.nodeStart : '#4a90d9';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        // Person icon (head + body)
        ctx.beginPath();
        ctx.arc(cx + r, cy - 2, 3, 0, Math.PI * 2);
        ctx.fillStyle = runtimeActive ? theme.nodeStart : '#4a90d9';
        ctx.fill();
        ctx.beginPath();
        ctx.moveTo(cx + r - 4, cy + 5);
        ctx.quadraticCurveTo(cx + r, cy + 1, cx + r + 4, cy + 5);
        ctx.stroke();
        break;
      }
      case 'item': {
        // Diamond
        ctx.beginPath();
        ctx.moveTo(cx + r, cy - r + 2);
        ctx.lineTo(cx + r + r - 2, cy);
        ctx.lineTo(cx + r, cy + r - 2);
        ctx.lineTo(cx + r - r + 2, cy);
        ctx.closePath();
        ctx.fillStyle = runtimeActive ? theme.nodeStart + '40' : '#e6a23c' + '30';
        ctx.fill();
        ctx.strokeStyle = runtimeActive ? theme.nodeStart : '#e6a23c';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        break;
      }
      case 'lock': {
        // Shield
        ctx.beginPath();
        ctx.moveTo(cx + r, cy - r + 2);
        ctx.quadraticCurveTo(cx + r + r - 1, cy - r + 4, cx + r + r - 2, cy);
        ctx.quadraticCurveTo(cx + r + r - 1, cy + r - 3, cx + r, cy + r - 1);
        ctx.quadraticCurveTo(cx + r - r + 1, cy + r - 3, cx + r - r + 2, cy);
        ctx.quadraticCurveTo(cx + r - r + 1, cy - r + 4, cx + r, cy - r + 2);
        ctx.closePath();
        ctx.fillStyle = runtimeActive ? theme.nodeStart + '40' : '#e74c3c' + '30';
        ctx.fill();
        ctx.strokeStyle = runtimeActive ? theme.nodeStart : '#e74c3c';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        break;
      }
      case 'container': {
        // Rounded rect
        roundRect(ctx, cx + 1, cy - r + 2, ENTITY_ICON_SIZE - 2, ENTITY_ICON_SIZE - 4, 3);
        ctx.fillStyle = runtimeActive ? theme.nodeStart + '40' : '#9b59b6' + '30';
        ctx.fill();
        ctx.strokeStyle = runtimeActive ? theme.nodeStart : '#9b59b6';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        break;
      }
      case 'scenery': {
        // Triangle
        ctx.beginPath();
        ctx.moveTo(cx + r, cy - r + 3);
        ctx.lineTo(cx + ENTITY_ICON_SIZE - 2, cy + r - 2);
        ctx.lineTo(cx + 2, cy + r - 2);
        ctx.closePath();
        ctx.fillStyle = '#27ae60' + '30';
        ctx.fill();
        ctx.strokeStyle = '#27ae60';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        break;
      }
      case 'property': {
        // Small filled square
        const s = ENTITY_ICON_SIZE - 6;
        ctx.fillStyle = runtimeActive ? theme.nodeStart + '40' : '#6c8ebf' + '30';
        ctx.fillRect(cx + r - s / 2, cy - s / 2, s, s);
        ctx.strokeStyle = runtimeActive ? theme.nodeStart : '#6c8ebf';
        ctx.lineWidth = 1.5;
        ctx.strokeRect(cx + r - s / 2, cy - s / 2, s, s);
        break;
      }
      default: {
        // Simple circle
        ctx.beginPath();
        ctx.arc(cx + r, cy, r - 1, 0, Math.PI * 2);
        ctx.fillStyle = theme.textMuted + '20';
        ctx.fill();
        ctx.strokeStyle = theme.textMuted;
        ctx.lineWidth = 1;
        ctx.stroke();
        break;
      }
    }
    ctx.restore();
  }

  function drawExitBadge(loc: WorldLocation, exit: WorldExit): void {
    if (!ctx || !theme) return;
    const pos = getExitEdgePosition(loc, exit.direction);
    if (!pos) return;

    const label = exit.direction.slice(0, 1).toUpperCase();
    ctx.font = 'bold 8px Outfit, system-ui, sans-serif';
    ctx.fillStyle = exit.isConditional ? theme.edgeConditional : theme.borderZone;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    // Small circular badge
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, 8, 0, Math.PI * 2);
    ctx.fillStyle = theme.bgPrimary;
    ctx.fill();
    ctx.strokeStyle = exit.isConditional ? theme.edgeConditional : theme.borderZone;
    ctx.lineWidth = 1;
    ctx.stroke();

    ctx.fillStyle = exit.isConditional ? theme.edgeConditional : theme.textMuted;
    ctx.fillText(label, pos.x, pos.y + 0.5);
  }

  function drawExit(loc: WorldLocation, exit: WorldExit): void {
    if (!ctx || !theme) return;
    const target = locationCenters.get(exit.targetId);
    if (!target) return;
    const source = locationCenters.get(loc.id);
    if (!source) return;

    const isVisited = runtimeOverlay?.visitedExitKeys.has(`${loc.id}/${exit.direction}`) ?? false;

    // Compute edge attachment points on card borders
    const from = getExitEdgePosition(loc, exit.direction) ?? { x: source.x, y: source.y };
    const to = getEntryPoint(target, from);

    // Curved arrow
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const curvature = Math.min(40, dist * 0.15);
    const mx = (from.x + to.x) / 2 - (dy / dist) * curvature;
    const my = (from.y + to.y) / 2 + (dx / dist) * curvature;

    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.quadraticCurveTo(mx, my, to.x, to.y);

    if (exit.isConditional) {
      ctx.setLineDash([6, 4]);
    } else {
      ctx.setLineDash([]);
    }

    ctx.strokeStyle = isVisited
      ? theme.nodeStart
      : exit.isConditional
        ? theme.edgeConditional
        : theme.edgeDefault;
    ctx.lineWidth = isVisited ? 2.5 : 1.5;
    ctx.globalAlpha = isVisited ? 1 : 0.7;
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.globalAlpha = 1;

    // Arrowhead at target end
    const angle = Math.atan2(to.y - my, to.x - mx);
    ctx.beginPath();
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(
      to.x - EXIT_ARROW_SIZE * Math.cos(angle - 0.4),
      to.y - EXIT_ARROW_SIZE * Math.sin(angle - 0.4),
    );
    ctx.lineTo(
      to.x - EXIT_ARROW_SIZE * Math.cos(angle + 0.4),
      to.y - EXIT_ARROW_SIZE * Math.sin(angle + 0.4),
    );
    ctx.closePath();
    ctx.fillStyle = isVisited
      ? theme.nodeStart
      : exit.isConditional
        ? theme.edgeConditional
        : theme.edgeDefault;
    ctx.fill();

    // Condition label
    if (exit.conditionLabel && zoom > 0.5) {
      ctx.font = '9px "JetBrains Mono", monospace';
      ctx.fillStyle = theme.edgeConditional;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'bottom';
      const labelText = `? ${exit.conditionLabel}`;
      // Background pill
      const tw = ctx.measureText(labelText).width + 8;
      ctx.fillStyle = theme.bgPrimary + 'E0';
      roundRect(ctx, mx - tw / 2, my - 14, tw, 14, 3);
      ctx.fill();
      ctx.fillStyle = theme.edgeConditional;
      ctx.fillText(labelText, mx, my - 2);
    }

    // Direction label at midpoint
    if (zoom > 0.6 && !exit.conditionLabel) {
      ctx.font = '9px Outfit, system-ui, sans-serif';
      ctx.fillStyle = theme.textMuted;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'bottom';
      ctx.fillText(exit.direction, mx, my - 4);
    }
  }

  function drawScene(scene: WorldScene): void {
    if (!ctx || !theme) return;

    // Hexagonal node for scenes
    const r = SCENE_RADIUS;
    ctx.beginPath();
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i - Math.PI / 6;
      const px = scene.x + r * Math.cos(angle);
      const py = scene.y + r * Math.sin(angle);
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.closePath();
    ctx.fillStyle = theme.accentPrimary + '15';
    ctx.fill();
    ctx.strokeStyle = theme.accentPrimary + '60';
    ctx.lineWidth = 1.5;
    ctx.stroke();

    // Scene label
    ctx.fillStyle = theme.accentPrimary;
    ctx.font = 'bold 10px Outfit, system-ui, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(truncate(scene.id, 12), scene.x, scene.y);

    // Choice branches
    if (zoom > 0.5 && scene.choices.length > 0) {
      const startY = scene.y + r + 8;
      for (let i = 0; i < scene.choices.length; i++) {
        const choice = scene.choices[i];
        const cy = startY + i * (CHOICE_H + 4);

        // Branch line from scene
        ctx.beginPath();
        ctx.moveTo(scene.x, scene.y + r);
        ctx.lineTo(scene.x, cy + CHOICE_H / 2);
        ctx.strokeStyle = choice.sticky ? theme.edgeChoice : theme.edgeChoice + '80';
        ctx.lineWidth = 1;
        if (!choice.sticky) ctx.setLineDash([3, 3]);
        ctx.stroke();
        ctx.setLineDash([]);

        // Choice pill
        const pillText = truncate(choice.label, 20);
        ctx.font = '9px Outfit, system-ui, sans-serif';
        const pillW = Math.max(80, ctx.measureText(pillText).width + 16);
        const pillX = scene.x - pillW / 2;

        ctx.fillStyle = theme.bgPrimary;
        roundRect(ctx, pillX, cy, pillW, CHOICE_H, 4);
        ctx.fill();
        ctx.strokeStyle = choice.sticky ? theme.edgeChoice : theme.edgeChoice + '60';
        ctx.lineWidth = 1;
        roundRect(ctx, pillX, cy, pillW, CHOICE_H, 4);
        ctx.stroke();

        ctx.fillStyle = theme.textPrimary;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(pillText, scene.x, cy + CHOICE_H / 2);

        // Effect count badge
        if (choice.effectCount > 0) {
          const badge = `${choice.effectCount} mut`;
          ctx.font = '8px "JetBrains Mono", monospace';
          ctx.fillStyle = theme.edgeConditional;
          ctx.textAlign = 'left';
          ctx.fillText(badge, pillX + pillW + 4, cy + CHOICE_H / 2);
        }
      }
    }
  }

  // ===== Hit testing =====

  function screenToWorld(sx: number, sy: number): { x: number; y: number } {
    return {
      x: (sx - canvasW / 2) / zoom - camX,
      y: (sy - canvasH / 2) / zoom - camY,
    };
  }

  function hitTestLocation(wx: number, wy: number): WorldLocation | null {
    for (const loc of data.locations) {
      if (wx >= loc.x && wx <= loc.x + loc.width && wy >= loc.y && wy <= loc.y + loc.height) {
        return loc;
      }
    }
    return null;
  }

  /** Hit test the header bar of a location card (drag handle area). */
  function hitTestLocationHeader(wx: number, wy: number): WorldLocation | null {
    for (const loc of data.locations) {
      if (wx >= loc.x && wx <= loc.x + loc.width && wy >= loc.y && wy <= loc.y + LOC_HEADER_H) {
        return loc;
      }
    }
    return null;
  }

  function hitTestEntity(wx: number, wy: number): { entity: WorldEntity; location: WorldLocation } | null {
    for (const loc of data.locations) {
      if (loc.entities.length === 0) continue;
      const entityStartY = loc.y + LOC_HEADER_H + LOC_DESC_H + 8;
      const cols = Math.max(1, Math.floor((loc.width - LOC_PADDING) / ENTITY_COL_W));

      for (let i = 0; i < loc.entities.length; i++) {
        const entity = loc.entities[i];
        const col = i % cols;
        const row = Math.floor(i / cols);
        const ex = loc.x + LOC_PADDING + col * ENTITY_COL_W;
        const ey = entityStartY + row * ENTITY_ROW_H;

        if (wx >= ex && wx <= ex + ENTITY_COL_W && wy >= ey && wy <= ey + ENTITY_ROW_H) {
          return { entity, location: loc };
        }
      }
    }
    return null;
  }

  // ===== Geometry helpers =====

  function getExitEdgePosition(loc: WorldLocation, direction: string): { x: number; y: number } | null {
    const d = direction.toLowerCase();
    const cx = loc.x + loc.width / 2;
    const cy = loc.y + loc.height / 2;

    switch (d) {
      case 'north': case 'up': return { x: cx, y: loc.y };
      case 'south': case 'down': return { x: cx, y: loc.y + loc.height };
      case 'east': return { x: loc.x + loc.width, y: cy };
      case 'west': return { x: loc.x, y: cy };
      case 'northeast': return { x: loc.x + loc.width, y: loc.y };
      case 'northwest': return { x: loc.x, y: loc.y };
      case 'southeast': return { x: loc.x + loc.width, y: loc.y + loc.height };
      case 'southwest': return { x: loc.x, y: loc.y + loc.height };
      default: return { x: cx, y: loc.y + loc.height };
    }
  }

  function getEntryPoint(
    target: { x: number; y: number; w: number; h: number },
    from: { x: number; y: number },
  ): { x: number; y: number } {
    // Find intersection of line from→target.center with target rect border
    const cx = target.x;
    const cy = target.y;
    const hw = target.w / 2;
    const hh = target.h / 2;
    const dx = from.x - cx;
    const dy = from.y - cy;

    if (dx === 0 && dy === 0) return { x: cx, y: cy - hh };

    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);

    if (absDx * hh > absDy * hw) {
      // Hits left or right
      const side = dx > 0 ? 1 : -1;
      return { x: cx + side * hw, y: cy + (dy / absDx) * hw };
    } else {
      // Hits top or bottom
      const side = dy > 0 ? 1 : -1;
      return { x: cx + (dx / absDy) * hh, y: cy + side * hh };
    }
  }

  function roundRect(c: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number): void {
    c.beginPath();
    c.moveTo(x + r, y);
    c.lineTo(x + w - r, y);
    c.arcTo(x + w, y, x + w, y + r, r);
    c.lineTo(x + w, y + h - r);
    c.arcTo(x + w, y + h, x + w - r, y + h, r);
    c.lineTo(x + r, y + h);
    c.arcTo(x, y + h, x, y + h - r, r);
    c.lineTo(x, y + r);
    c.arcTo(x, y, x + r, y, r);
    c.closePath();
  }

  function truncate(s: string, max: number): string {
    return s.length > max ? s.slice(0, max - 1) + '\u2026' : s;
  }

  // ===== Event handlers =====

  function handlePointerDown(e: PointerEvent): void {
    if (e.button !== 0) return;
    (e.target as Element)?.setPointerCapture?.(e.pointerId);

    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    const world = screenToWorld(sx, sy);

    panStartX = e.clientX;
    panStartY = e.clientY;
    pointerDownWorldX = world.x;
    pointerDownWorldY = world.y;

    // Check if we hit a location card header (drag handle)
    const locHit = hitTestLocationHeader(world.x, world.y);
    if (locHit) {
      isDraggingNode = true;
      draggedLocation = locHit;
      dragOffsetX = world.x - locHit.x;
      dragOffsetY = world.y - locHit.y;
      if (canvasEl) canvasEl.style.cursor = 'grabbing';
      return;
    }

    // Otherwise pan
    isPanning = true;
    panOriginCamX = camX;
    panOriginCamY = camY;
  }

  function handlePointerMove(e: PointerEvent): void {
    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;

    // Dragging a location card
    if (isDraggingNode && draggedLocation) {
      const world = screenToWorld(sx, sy);
      draggedLocation.x = world.x - dragOffsetX;
      draggedLocation.y = world.y - dragOffsetY;
      // Update location centre cache
      locationCenters.set(draggedLocation.id, {
        x: draggedLocation.x + draggedLocation.width / 2,
        y: draggedLocation.y + draggedLocation.height / 2,
        w: draggedLocation.width,
        h: draggedLocation.height,
      });
      return;
    }

    // Panning
    if (isPanning) {
      camX = panOriginCamX + (e.clientX - panStartX) / zoom;
      camY = panOriginCamY + (e.clientY - panStartY) / zoom;
      return;
    }

    // Hover hit test
    const world = screenToWorld(sx, sy);
    const entityHit = hitTestEntity(world.x, world.y);
    if (entityHit) {
      hoveredEntityId = entityHit.entity.id;
      hoveredLocationId = entityHit.location.id;
      tooltipEntity = entityHit.entity;
      tooltipX = e.clientX;
      tooltipY = e.clientY;
      if (canvasEl) canvasEl.style.cursor = 'pointer';
      return;
    }

    hoveredEntityId = null;
    tooltipEntity = null;

    const locHit = hitTestLocation(world.x, world.y);
    hoveredLocationId = locHit?.id ?? null;
    const headerHit = hitTestLocationHeader(world.x, world.y);
    if (canvasEl) canvasEl.style.cursor = headerHit ? 'grab' : locHit ? 'pointer' : 'default';
  }

  function handlePointerUp(e: PointerEvent): void {
    const wasDragging = isDraggingNode;
    const draggedLoc = draggedLocation;
    isDraggingNode = false;
    draggedLocation = null;
    isPanning = false;
    if (canvasEl) canvasEl.style.cursor = 'default';

    // Notify parent when a node was dragged to a new position
    if (wasDragging && draggedLoc && onNodeMoved) {
      onNodeMoved(draggedLoc.id, draggedLoc.x, draggedLoc.y);
    }

    // Click detection (minimal movement)
    const moved = Math.abs(e.clientX - panStartX) + Math.abs(e.clientY - panStartY);
    if (moved < 4 && !wasDragging) {
      if (!canvasEl) return;
      const rect = canvasEl.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;
      const world = screenToWorld(sx, sy);

      const now = Date.now();

      // Entity hit
      const entityHit = hitTestEntity(world.x, world.y);
      if (entityHit) {
        const targetKey = `entity:${entityHit.entity.id}`;
        if (now - lastClickTime < DBLCLICK_MS && lastClickTarget === targetKey) {
          // Double-click → navigate to code
          if (onEntityDblClick) onEntityDblClick(entityHit.entity.id);
          lastClickTime = 0;
          lastClickTarget = null;
        } else {
          // Single click → select
          if (onEntityClick) onEntityClick(entityHit.entity.id);
          lastClickTime = now;
          lastClickTarget = targetKey;
        }
        return;
      }

      // Location hit
      const locHit = hitTestLocation(world.x, world.y);
      if (locHit) {
        const targetKey = `loc:${locHit.id}`;
        if (now - lastClickTime < DBLCLICK_MS && lastClickTarget === targetKey) {
          // Double-click → navigate to code
          if (onLocationDblClick) onLocationDblClick(locHit.id);
          lastClickTime = 0;
          lastClickTarget = null;
        } else {
          // Single click → select
          if (onLocationClick) onLocationClick(locHit.id);
          lastClickTime = now;
          lastClickTarget = targetKey;
        }
        return;
      }

      // Clicked empty space
      lastClickTime = 0;
      lastClickTarget = null;
    }
  }

  function handleWheel(e: WheelEvent): void {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.92 : 1.08;
    const newZoom = Math.max(0.1, Math.min(20, zoom * factor));

    // Zoom toward cursor
    if (canvasEl) {
      const rect = canvasEl.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;
      const wx = (sx - canvasW / 2) / zoom - camX;
      const wy = (sy - canvasH / 2) / zoom - camY;
      camX = (sx - canvasW / 2) / newZoom - wx;
      camY = (sy - canvasH / 2) / newZoom - wy;
    }

    zoom = newZoom;
  }

  function fitToView(): void {
    centreOnWorld();
  }

  function zoomIn(): void {
    zoom = Math.min(20, zoom * 1.3);
  }

  function zoomOut(): void {
    zoom = Math.max(0.1, zoom * 0.7);
  }
</script>

<div class="world-canvas" bind:this={containerEl}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <canvas
    bind:this={canvasEl}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onwheel={handleWheel}
  ></canvas>

  <!-- Empty state overlay -->
  {#if data.locations.length === 0}
    <div class="world-canvas__empty">
      <p>{emptyMessage}</p>
      <p class="world-canvas__hint">Data appears after successful compilation</p>
    </div>
  {/if}

  <!-- Entity tooltip -->
  {#if tooltipEntity && zoom > 0.3}
    <div
      class="world-canvas__tooltip"
      style="left: {tooltipX + 12}px; top: {tooltipY - 8}px;"
    >
      <div class="world-canvas__tooltip-name">{tooltipEntity.name}</div>
      {#if tooltipEntity.type}
        <div class="world-canvas__tooltip-type">{tooltipEntity.type}</div>
      {/if}
      {#each Object.entries(tooltipEntity.properties).slice(0, 6) as [key, value]}
        <div class="world-canvas__tooltip-prop">
          <span class="world-canvas__tooltip-key">{key}</span>
          <span class="world-canvas__tooltip-val">{String(value)}</span>
        </div>
      {/each}
      {#if Object.keys(tooltipEntity.properties).length > 6}
        <div class="world-canvas__tooltip-more">
          +{Object.keys(tooltipEntity.properties).length - 6} more
        </div>
      {/if}
    </div>
  {/if}

  {#if data.locations.length > 0}
    <!-- Toolbar -->
    <div class="world-canvas__toolbar">
      <button onclick={fitToView} title="Fit to view">⊞</button>
      <button onclick={zoomIn} title="Zoom in">+</button>
      <button onclick={zoomOut} title="Zoom out">−</button>
      <span class="world-canvas__zoom">{Math.round(zoom * 100)}%</span>
    </div>

    <!-- Filters -->
    <div class="world-canvas__filters">
      <label class="world-canvas__filter">
        <input type="checkbox" bind:checked={showEntities} /> Entities
      </label>
      <label class="world-canvas__filter">
        <input type="checkbox" bind:checked={showExits} /> Exits
      </label>
      <label class="world-canvas__filter">
        <input type="checkbox" bind:checked={showScenes} /> Scenes
      </label>
    </div>

    <!-- Legend -->
    <div class="world-canvas__legend">
      <span class="world-canvas__legend-item"><span class="world-canvas__legend-icon world-canvas__legend-icon--character">●</span> Character</span>
      <span class="world-canvas__legend-item"><span class="world-canvas__legend-icon world-canvas__legend-icon--item">◆</span> Item</span>
      <span class="world-canvas__legend-item"><span class="world-canvas__legend-icon world-canvas__legend-icon--lock">◇</span> Lock</span>
      <span class="world-canvas__legend-item"><span class="world-canvas__legend-icon world-canvas__legend-icon--container">▪</span> Container</span>
    </div>

    <!-- Runtime status -->
    {#if runtimeOverlay?.isPlaying}
      <div class="world-canvas__runtime-badge">
        ▸ Turn {runtimeOverlay.turnCount}
      </div>
    {/if}

    <!-- World name -->
    {#if data.worldName}
      <div class="world-canvas__title">{data.worldName}</div>
    {/if}
  {/if}
</div>

<style>
  .world-canvas {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
  }

  .world-canvas canvas {
    display: block;
    cursor: grab;
  }

  .world-canvas canvas:active {
    cursor: grabbing;
  }

  .world-canvas__empty {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
    gap: var(--forge-space-sm);
    pointer-events: none;
  }

  .world-canvas__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  /* Tooltip */
  .world-canvas__tooltip {
    position: fixed;
    z-index: 9999;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 8px 10px;
    pointer-events: none;
    max-width: 220px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .world-canvas__tooltip-name {
    font-weight: 600;
    font-size: 12px;
    color: var(--forge-text-primary);
    margin-bottom: 2px;
  }

  .world-canvas__tooltip-type {
    font-size: 10px;
    color: var(--forge-accent-primary);
    margin-bottom: 4px;
    font-family: var(--forge-font-family-mono);
  }

  .world-canvas__tooltip-prop {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 10px;
    line-height: 1.6;
  }

  .world-canvas__tooltip-key {
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .world-canvas__tooltip-val {
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-weight: 600;
  }

  .world-canvas__tooltip-more {
    font-size: 9px;
    color: var(--forge-text-muted);
    margin-top: 2px;
  }

  /* Toolbar */
  .world-canvas__toolbar {
    position: absolute;
    top: var(--forge-space-sm);
    right: var(--forge-space-sm);
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 2px;
  }

  .world-canvas__toolbar button {
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 14px;
    cursor: pointer;
    border-radius: var(--forge-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .world-canvas__toolbar button:hover {
    background: var(--forge-bg-hover);
    color: var(--forge-text-primary);
  }

  .world-canvas__zoom {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
    padding: 0 4px;
    min-width: 36px;
    text-align: center;
  }

  /* Filters */
  .world-canvas__filters {
    position: absolute;
    bottom: var(--forge-space-sm);
    left: var(--forge-space-sm);
    display: flex;
    gap: var(--forge-space-sm);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 4px 8px;
  }

  .world-canvas__filter {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--forge-text-muted);
    cursor: pointer;
    user-select: none;
  }

  .world-canvas__filter input {
    accent-color: var(--forge-accent-primary);
    width: 12px;
    height: 12px;
  }

  /* Legend */
  .world-canvas__legend {
    position: absolute;
    bottom: var(--forge-space-sm);
    right: var(--forge-space-sm);
    display: flex;
    gap: var(--forge-space-md);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 4px 10px;
  }

  .world-canvas__legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .world-canvas__legend-icon {
    font-size: 10px;
  }

  .world-canvas__legend-icon--character { color: #4a90d9; }
  .world-canvas__legend-icon--item { color: #e6a23c; }
  .world-canvas__legend-icon--lock { color: #e74c3c; }
  .world-canvas__legend-icon--container { color: #9b59b6; }

  /* Runtime badge */
  .world-canvas__runtime-badge {
    position: absolute;
    top: var(--forge-space-sm);
    left: var(--forge-space-sm);
    background: var(--forge-runtime-play-active, #4caf50);
    color: var(--forge-bg-primary);
    font-size: 10px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: var(--forge-radius-sm);
  }

  /* Title */
  .world-canvas__title {
    position: absolute;
    top: var(--forge-space-sm);
    left: 50%;
    transform: translateX(-50%);
    font-size: 11px;
    font-weight: 600;
    color: var(--forge-text-muted);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 2px 10px;
    pointer-events: none;
  }
</style>
