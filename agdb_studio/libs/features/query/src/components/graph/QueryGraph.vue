<script lang="ts" setup>
import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import { onMounted, onUnmounted, ref, watch } from "vue";
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
const nodeColor = (node: object) =>
  getNodeColor(asGraphNode(node), highlight, theme);
const linkColor = (link: object) =>
  getLinkColor(asGraphLink(link), highlight, theme);
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

const onNodeClick = (node: object) => {
  actions.clickNode(asGraphNode(node));
};

const onNodeRightClick = (node: object, event: MouseEvent) => {
  event.preventDefault?.();
  actions.openMenu(asGraphNode(node), event.clientX, event.clientY);
};

const onLinkClick = (link: object) => {
  actions.closeMenu();
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

onMounted(() => {
  /* v8 ignore else -- @preserve */
  if (graphContainer.value) {
    // Keep the selection when the graph is re-mounted (e.g. after the table view).
    highlight.selectedId = actions.selectedId.value;
    highlight.pendingSourceId = actions.pendingEdgeSourceId.value;
    setData(graphData.value);
    theme = readGraphTheme(graphContainer.value);
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
      .linkOpacity(0.85)
      .linkColor(linkColor)
      .linkWidth(linkWidth)
      .linkCurvature(linkCurvature)
      .linkDirectionalArrowLength(4)
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
    if (added) {
      clearTimeout(zoomTimer);
      graphInstance.d3ReheatSimulation();
      scheduleZoomToFit();
    }
  }
});

watch(
  () => [actions.selectedId.value, actions.pendingEdgeSourceId.value],
  ([selectedId, pendingSourceId]) => {
    highlight = { ...highlight, selectedId, pendingSourceId };
    restyle();
  },
);

onUnmounted(() => {
  clearTimeout(zoomTimer);
  colorSchemeQuery?.removeEventListener?.("change", applyTheme);
  resizeObserver?.disconnect();
  /* v8 ignore else -- @preserve */
  if (graphInstance) {
    graphInstance._destructor();
    graphInstance = undefined;
  }
});
</script>

<template>
  <div
    ref="graphContainer"
    class="query-graph"
    :class="{ hovering: isHovering }"
    data-testid="query-graph"
    @contextmenu.prevent
  ></div>
</template>

<style lang="less" scoped>
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

  :deep(.graph-tooltip-more) {
    color: var(--color-text-muted);
    font-style: italic;
  }
}
</style>
