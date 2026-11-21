<script lang="ts" setup>
import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import { onMounted, onUnmounted, ref, useId, watch } from "vue";
import PinnedCard from "./PinnedCard.vue";
import type { GraphNode, GraphLink, GraphData } from "../../composables/types";
import { useGraphActions } from "../../composables/graphActions";
import { carryOverLayout } from "../../utils/graphLayout";
import {
  GRAPH_THEME_FALLBACK,
  computeHighlight,
  computeLinkCurvatures,
  emptyHighlightState,
  getLinkColor,
  getLinkWidth,
  getNodeColor,
  linkTooltip,
  nodeTooltip,
  readGraphTheme,
  type GraphTheme,
  type HighlightState,
} from "../../utils/graphStyle";
import { slotColor } from "../../utils/graphColorRules";
import {
  createLinkGradientMaterials,
  GRADIENT_MAX_LINKS,
} from "../../utils/linkGradient";

/**
 * 3d-force-graph uses generic node/link objects internally.
 * These helpers cast to our typed variants to avoid repeated `as unknown as` chains.
 */
/* v8 ignore next 2 -- @preserve */
const asGraphNode = (node: object): GraphNode => node as GraphNode;
const asGraphLink = (link: object): GraphLink => link as unknown as GraphLink;

const graphContainer = ref<HTMLElement | null>(null);

const injectedActions = useGraphActions();
if (!injectedActions) {
  // Fail loudly: a private instance would not be connected to the results UI.
  throw new Error("QueryGraph must be rendered inside QueryResults");
}
const actions = injectedActions;
const { graphData } = actions;

let theme: GraphTheme = GRAPH_THEME_FALLBACK;
let highlight: HighlightState = emptyHighlightState();
let curvatures = new Map<number, number>();
let currentLinks: GraphLink[] = [];

const isHovering = ref(false);

let graphInstance: ForceGraph3DInstance | undefined;
let zoomTimer: ReturnType<typeof setTimeout> | undefined;
let colorSchemeQuery: MediaQueryList | undefined;
let resizeObserver: ResizeObserver | undefined;

/* v8 ignore start -- @preserve: accessors are invoked by the WebGL renderer */
/** The colour a colour rule gives the element, if any. */
const customColor = (element: GraphNode | GraphLink): string | undefined => {
  const slot = actions.colorSlotOf(element);
  return slot === undefined ? undefined : slotColor(theme.palette, slot);
};
const nodeColor = (node: object) =>
  getNodeColor(
    asGraphNode(node),
    highlight,
    theme,
    customColor(asGraphNode(node)),
    actions.colorLegend.value.length > 0,
  );
const linkColor = (link: object) =>
  getLinkColor(
    asGraphLink(link),
    highlight,
    theme,
    customColor(asGraphLink(link)),
  );
/** Links darken towards their source so the direction reads at a glance. */
const LINK_OPACITY = 0.85;
const gradients = createLinkGradientMaterials(LINK_OPACITY);
const linkMaterial = (link: object) =>
  currentLinks.length > GRADIENT_MAX_LINKS
    ? null
    : gradients.material(linkColor(link), linkCurvature(link) !== 0);
const linkWidth = (link: object) => getLinkWidth(asGraphLink(link), highlight);
const linkCurvature = (link: object) =>
  curvatures.get(asGraphLink(link).id) ?? 0;
const linkParticles = (link: object) =>
  asGraphLink(link).id === highlight.hoveredLinkId ? 3 : 0;
/* v8 ignore stop -- @preserve */

/** Re-applies the style accessors so the renderer picks up the new state. */
const restyle = () => {
  if (!graphInstance) return;
  graphInstance
    .nodeColor(nodeColor)
    .linkColor(linkColor)
    .linkMaterial(linkMaterial)
    .linkWidth(linkWidth)
    .linkDirectionalArrowColor(linkColor)
    .linkDirectionalParticles(linkParticles);
};

const setHighlight = (hovered: { node?: GraphNode; link?: GraphLink }) => {
  highlight = {
    ...highlight,
    hoveredNodeId: hovered.node?.id,
    hoveredLinkId: hovered.link?.id,
    ...computeHighlight(hovered, currentLinks),
  };
  isHovering.value = !!(hovered.node || hovered.link);
  restyle();
};

const onNodeHover = (node: object | null) => {
  setHighlight({ node: node ? asGraphNode(node) : undefined });
};

const onLinkHover = (link: object | null) => {
  setHighlight({ link: link ? asGraphLink(link) : undefined });
};

const onNodeClick = (node: object, event?: MouseEvent) => {
  actions.clickNode(asGraphNode(node), { pin: !!event?.shiftKey });
};

const onNodeRightClick = (node: object, event: MouseEvent) => {
  event.preventDefault?.();
  actions.openMenu(asGraphNode(node), event.clientX, event.clientY);
};

const onLinkClick = (link: object, event?: MouseEvent) => {
  actions.closeMenu();
  if (event?.shiftKey) {
    actions.togglePin(asGraphLink(link));
    return;
  }
  actions.viewElement(asGraphLink(link));
};

const onLinkRightClick = (link: object, event: MouseEvent) => {
  event.preventDefault?.();
  actions.openMenu(asGraphLink(link), event.clientX, event.clientY);
};

const onBackgroundClick = () => {
  actions.dismiss();
};

const onBackgroundRightClick = () => {
  actions.closeMenu();
};

const applyTheme = () => {
  if (!graphContainer.value || !graphInstance) return;
  theme = readGraphTheme(graphContainer.value);
  actions.paletteSize.value = theme.palette.length;
  graphInstance.backgroundColor(theme.background);
  restyle();
};

/** The nodes passed to the graph last; 3d-force-graph keeps their layout on them. */
let shownNodes: GraphNode[] = [];

const setData = (data: GraphData) => {
  shownNodes = data.nodes;
  currentLinks = data.links;
  curvatures = computeLinkCurvatures(data.links);
  highlight = {
    ...emptyHighlightState(),
    selectedId: highlight.selectedId,
    pendingSourceId: highlight.pendingSourceId,
  };
  isHovering.value = false;
};

/** Padding (px) kept around the graph when zooming to fit. */
const FIT_PADDING = 40;
let fitOnEngineStop = false;

/**
 * Zooms to fit shortly after new data arrives and once more when the layout
 * simulation has settled, so nodes that drifted are not cut off.
 */
const scheduleZoomToFit = () => {
  clearTimeout(zoomTimer);
  fitOnEngineStop = true;
  zoomTimer = setTimeout(() => graphInstance?.zoomToFit(400), 300);
};

const onEngineStop = () => {
  if (!fitOnEngineStop) return;
  fitOnEngineStop = false;
  graphInstance?.zoomToFit(400, FIT_PADDING);
};

type Point = { x: number; y: number };
type Positioned = { x?: number; y?: number; z?: number };
/** The library adds coordinates to the node objects it lays out. */
const isPositioned = (value: unknown): value is Positioned =>
  typeof value === "object" && value !== null;

/** Screen position (px, relative to the graph) of a node or an edge's middle. */
const elementPoint = (id: number): Point | undefined => {
  /* v8 ignore if -- @preserve */
  if (!graphInstance) return undefined;
  let position: Positioned | undefined;
  if (id > 0) {
    const node: unknown = shownNodes.find((n) => n.id === id);
    if (isPositioned(node)) position = node;
  } else {
    const link = currentLinks.find((l) => l.id === id);
    // The library replaces the endpoint ids with the node objects.
    const from: unknown = link?.source;
    const to: unknown = link?.target;
    if (isPositioned(from) && isPositioned(to)) {
      position = {
        x: ((from.x ?? 0) + (to.x ?? 0)) / 2,
        y: ((from.y ?? 0) + (to.y ?? 0)) / 2,
        z: ((from.z ?? 0) + (to.z ?? 0)) / 2,
      };
    }
  }
  if (position?.x === undefined) return undefined;
  return graphInstance.graph2ScreenCoords(
    position.x,
    position.y ?? 0,
    position.z ?? 0,
  );
};

/** Where pinned elements are on screen; cards are drawn next to them. */
const pinAnchors = ref<Record<number, Point>>({});
/** Card position relative to its element, changed by dragging (kept with the pin). */
const DEFAULT_CARD_OFFSET: Point = { x: 18, y: -18 };
const cardOffset = (id: number): Point =>
  actions.cardOffset(id) ?? DEFAULT_CARD_OFFSET;

const samePoints = (a: Record<number, Point>, b: Record<number, Point>) => {
  const keys = Object.keys(a);
  return (
    keys.length === Object.keys(b).length &&
    keys.every((key) => {
      const p = a[Number(key)];
      const q = b[Number(key)];
      return !!p && !!q && p.x === q.x && p.y === q.y;
    })
  );
};

/** The line drawn from the add-edge source to the pointer (or target). */
const rubberBand = ref<{ from: Point; to: Point; snapped: boolean }>();
let pointer: Point | undefined;
const markerId = `graph-arrow-${useId()}`;

let frame: number | undefined;

/** Follows the layout and camera each frame while anything is attached. */
const track = () => {
  frame = undefined;
  const anchors: Record<number, Point> = {};
  for (const id of actions.pinnedIds.value) {
    const point = elementPoint(id);
    if (point) anchors[id] = point;
  }
  // Re-render the cards only when something moved.
  if (!samePoints(anchors, pinAnchors.value)) pinAnchors.value = anchors;

  const sourceId = actions.pendingEdgeSourceId.value;
  const from = sourceId === undefined ? undefined : elementPoint(sourceId);
  const targetId = highlight.hoveredNodeId;
  const target =
    targetId !== undefined && targetId !== sourceId
      ? elementPoint(targetId)
      : undefined;
  const to = target ?? pointer;
  rubberBand.value = from && to ? { from, to, snapped: !!target } : undefined;

  if (actions.pinnedIds.value.length || sourceId !== undefined) {
    frame = requestAnimationFrame(track);
  }
};

const startTracking = () => {
  if (frame === undefined) frame = requestAnimationFrame(track);
};

watch(
  () => [actions.pinnedIds.value.length, actions.pendingEdgeSourceId.value],
  () => startTracking(),
);

const onPointerMove = (event: PointerEvent) => {
  const rect = graphContainer.value?.getBoundingClientRect();
  /* v8 ignore if -- @preserve */
  if (!rect) return;
  pointer = { x: event.clientX - rect.left, y: event.clientY - rect.top };
};

const onPointerLeave = () => {
  pointer = undefined;
};

/** Ends a card drag still in progress, e.g. on unmount. */
let stopCardDrag: (() => void) | undefined;

/** Drags a pinned card; the offset to its element is kept afterwards. */
const startCardDrag = (id: number, event: PointerEvent) => {
  stopCardDrag?.();
  const start = { x: event.clientX, y: event.clientY };
  const startOffset = cardOffset(id);
  const move = (e: PointerEvent) => {
    actions.setCardOffset(id, {
      x: startOffset.x + e.clientX - start.x,
      y: startOffset.y + e.clientY - start.y,
    });
  };
  const stop = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", stop);
    window.removeEventListener("pointercancel", stop);
    stopCardDrag = undefined;
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", stop);
  window.addEventListener("pointercancel", stop);
  stopCardDrag = stop;
};

/** Briefly sends particles along a newly created edge to show its direction. */
const animateNewEdge = (edgeId: number) => {
  // The library keeps (and animates) the same link objects it was given.
  const link = currentLinks.find((l) => l.id === edgeId);
  if (!link || !graphInstance) return false;
  for (const delay of [0, 250, 500]) {
    setTimeout(() => graphInstance?.emitParticle(link), delay);
  }
  return true;
};

let edgeToAnimate: number | undefined;
watch(
  () => actions.lastAddedEdgeId.value,
  (edgeId) => {
    edgeToAnimate = edgeId;
    if (edgeId !== undefined && animateNewEdge(edgeId)) {
      edgeToAnimate = undefined;
    }
  },
);

onMounted(() => {
  /* v8 ignore else -- @preserve */
  if (graphContainer.value) {
    // Keep the selection when the graph is re-mounted (e.g. after the table view).
    highlight.selectedId = actions.selectedId.value;
    highlight.pendingSourceId = actions.pendingEdgeSourceId.value;
    setData(graphData.value);
    theme = readGraphTheme(graphContainer.value);
    actions.paletteSize.value = theme.palette.length;
    // Measure before the library inserts its (window sized) canvas.
    const { clientWidth, clientHeight } = graphContainer.value;

    graphInstance = new ForceGraph3D(graphContainer.value)
      .graphData(graphData.value)
      .backgroundColor(theme.background)
      .nodeRelSize(4)
      .nodeOpacity(0.95)
      .nodeColor(nodeColor)
      .nodeLabel((node) =>
        nodeTooltip(actions.describeElement(asGraphNode(node))),
      )
      .linkOpacity(LINK_OPACITY)
      .linkColor(linkColor)
      .linkMaterial(linkMaterial)
      .linkWidth(linkWidth)
      .linkCurvature(linkCurvature)
      .linkDirectionalArrowLength(6)
      .linkDirectionalArrowRelPos(1)
      .linkDirectionalArrowColor(linkColor)
      .linkDirectionalParticles(linkParticles)
      .linkDirectionalParticleWidth(2)
      .linkLabel((link) =>
        linkTooltip(actions.describeElement(asGraphLink(link))),
      )
      .onNodeHover(onNodeHover)
      .onLinkHover(onLinkHover)
      .onNodeClick(onNodeClick)
      .onNodeRightClick(onNodeRightClick)
      .onLinkClick(onLinkClick)
      .onLinkRightClick(onLinkRightClick)
      .onBackgroundClick(onBackgroundClick)
      .onBackgroundRightClick(onBackgroundRightClick)
      .onEngineStop(onEngineStop)
      .height(clientHeight)
      .width(clientWidth);

    // Follow container size changes (window resize, layout changes).
    if (typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(([entry]) => {
        if (!entry || !graphInstance) return;
        const { width, height } = entry.contentRect;
        if (width > 0 && height > 0) graphInstance.width(width).height(height);
      });
      resizeObserver.observe(graphContainer.value);
    }

    // The theme follows the OS color scheme; recolor the graph when it changes.
    colorSchemeQuery = window.matchMedia?.("(prefers-color-scheme: dark)");
    colorSchemeQuery?.addEventListener?.("change", applyTheme);

    if (graphData.value.nodes.length > 0) scheduleZoomToFit();
    startTracking();
  }
});

watch(graphData, (newData) => {
  /* v8 ignore next -- @preserve */
  if (graphInstance) {
    // Elements are rebuilt on every change (e.g. after editing a value), so
    // keep the existing layout and only settle and refit when nodes are added.
    const added = carryOverLayout(shownNodes, newData.nodes);
    setData(newData);
    graphInstance.graphData(newData);
    restyle();
    // The new edge arrives with the refreshed data.
    if (edgeToAnimate !== undefined && animateNewEdge(edgeToAnimate)) {
      edgeToAnimate = undefined;
    }
    if (added) {
      clearTimeout(zoomTimer);
      graphInstance.d3ReheatSimulation();
      scheduleZoomToFit();
    }
  }
});

// Colour rules changed, or the elements they apply to.
watch(
  () => actions.colorResolver.value,
  () => restyle(),
);

watch(
  () => [actions.selectedId.value, actions.pendingEdgeSourceId.value],
  ([selectedId, pendingSourceId]) => {
    highlight = { ...highlight, selectedId, pendingSourceId };
    restyle();
  },
);

onUnmounted(() => {
  stopCardDrag?.();
  clearTimeout(zoomTimer);
  if (frame !== undefined) cancelAnimationFrame(frame);
  colorSchemeQuery?.removeEventListener?.("change", applyTheme);
  resizeObserver?.disconnect();
  gradients.dispose();
  /* v8 ignore else -- @preserve */
  if (graphInstance) {
    graphInstance._destructor();
    graphInstance = undefined;
  }
});
</script>

<template>
  <div
    class="query-graph-wrapper"
    @pointermove="onPointerMove"
    @pointerleave="onPointerLeave"
  >
    <div
      ref="graphContainer"
      class="query-graph"
      :class="{
        hovering: isHovering,
        'adding-edge': actions.pendingEdgeSourceId.value !== undefined,
      }"
      data-testid="query-graph"
      @contextmenu.prevent
    ></div>
    <div class="graph-overlay" data-testid="graph-overlay">
      <svg class="overlay-lines" aria-hidden="true">
        <defs>
          <marker
            :id="markerId"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" class="arrow-head" />
          </marker>
        </defs>
        <template
          v-for="element in actions.pinnedElements.value"
          :key="element.id"
        >
          <line
            v-if="pinAnchors[element.id]"
            class="leader-line"
            :x1="pinAnchors[element.id]!.x"
            :y1="pinAnchors[element.id]!.y"
            :x2="pinAnchors[element.id]!.x + cardOffset(element.id).x"
            :y2="pinAnchors[element.id]!.y + cardOffset(element.id).y"
          />
        </template>
        <Transition name="rubber-band">
          <line
            v-if="rubberBand"
            class="rubber-band"
            :class="{ snapped: rubberBand.snapped }"
            data-testid="graph-rubber-band"
            :x1="rubberBand.from.x"
            :y1="rubberBand.from.y"
            :x2="rubberBand.to.x"
            :y2="rubberBand.to.y"
            :marker-end="`url(#${markerId})`"
          />
        </Transition>
      </svg>
      <template
        v-for="element in actions.pinnedElements.value"
        :key="element.id"
      >
        <PinnedCard
          v-if="pinAnchors[element.id]"
          :element="element"
          :x="pinAnchors[element.id]!.x + cardOffset(element.id).x"
          :y="pinAnchors[element.id]!.y + cardOffset(element.id).y"
          @unpin="actions.unpin(element.id)"
          @open="actions.viewElement(element)"
          @drag-start="
            (event: PointerEvent) => startCardDrag(element.id, event)
          "
        />
      </template>
    </div>
  </div>
</template>

<style lang="less" scoped>
.query-graph-wrapper {
  position: relative;
  width: 100%;
  min-width: 0;
  height: 100%;
  min-height: 20rem;
}
.graph-overlay {
  position: absolute;
  inset: 0;
  overflow: hidden;
  // Only the cards take the pointer; the graph below stays interactive.
  pointer-events: none;
}
.overlay-lines {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}
.leader-line {
  stroke: var(--color-text-muted);
  stroke-width: 1;
  stroke-dasharray: 2 3;
}
.rubber-band {
  stroke: var(--graph-selected-color);
  stroke-width: 2;
  stroke-dasharray: 6 4;
  animation: rubber-band-march 0.6s linear infinite;
  &.snapped {
    stroke-dasharray: none;
    animation: none;
  }
}
.arrow-head {
  fill: var(--graph-selected-color);
}
@keyframes rubber-band-march {
  to {
    stroke-dashoffset: -10;
  }
}
.rubber-band-leave-active {
  transition: opacity 0.25s ease;
}
.rubber-band-leave-to {
  opacity: 0;
}
.query-graph {
  position: relative;
  width: 100%;
  min-width: 0;
  height: 100%;
  min-height: 20rem;
  // Ignore the canvas' own size so the container (and the canvas with it) can shrink.
  contain: size;
  overflow: hidden;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--graph-background);

  &.hovering {
    cursor: pointer;
  }

  &.adding-edge {
    cursor: crosshair;
  }

  // Reset the library's default tooltip frame (class differs between versions).
  :deep(.scene-tooltip),
  :deep(.float-tooltip-kap) {
    padding: 0;
    background: none;
    border: none;
    box-shadow: none;
  }

  :deep(.graph-tooltip) {
    max-width: 20rem;
    padding: 0.4rem 0.6rem;
    border-radius: 0.4rem;
    border: 1px solid var(--color-border);
    background-color: var(--color-background-mute);
    color: var(--color-text);
    font-size: 0.8rem;
    line-height: 1.3;
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.2);
  }

  :deep(.graph-tooltip-title) {
    font-weight: bold;
    margin-bottom: 0.2rem;
  }

  :deep(.graph-tooltip-alias) {
    font-weight: normal;
    color: var(--color-text-muted);
  }

  :deep(.graph-tooltip-key) {
    color: var(--color-text-muted);
  }

  :deep(.graph-tooltip-tag) {
    font-size: 0.7rem;
    padding: 0 0.3rem;
    border-radius: 0.3rem;
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
  }

  :deep(.graph-tooltip-hint) {
    margin-top: 0.2rem;
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  :deep(.graph-tooltip-more) {
    color: var(--color-text-muted);
    font-style: italic;
  }
}
</style>
