import { errorMessage } from "@agdb-studio/api/src/apiError";
import type { DbKeyValue } from "@agnesoft/agdb_api";
import {
  computed,
  inject,
  provide,
  readonly,
  ref,
  watch,
  type InjectionKey,
  type Ref,
} from "vue";
import type { Action } from "@agdb-studio/common/src/composables/content/types";
import { convertArrayOfStringsToContent } from "@agdb-studio/common/src/composables/content/utils";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { useDbExec } from "./dbExec";
import { PREFETCH_LIMIT, useElementValues } from "./elementValues";
import { useGraphData } from "./graphData";
import { useQueryStore } from "./queryStore";
import { getQueryStepLabel } from "../mock/queryApiMock";
import type {
  GraphElement,
  GraphNode,
  PinnedDetail,
  ScreenPoint,
} from "./types";
import {
  deleteElementQuery,
  insertEdgeQuery,
  neighboursQueries,
  selectElementQuery,
  updateValuesQueries,
} from "../utils/graphActionQueries";
import {
  buildColorResolver,
  numericValue,
  ruleScale,
  unusedScale,
  type ColorRef,
  type ColorRule,
  type PaletteSlot,
} from "../utils/graphColorRules";
import { searchStart } from "../utils/graphDistance";
import { GRAPH_THEME_FALLBACK } from "../utils/graphStyle";

export type PanelMode = "view" | "edit";

export type ContextMenuState = {
  x: number;
  y: number;
  element: GraphElement;
};

export const isNode = (element: GraphElement): element is GraphNode =>
  element.id > 0;

/** E.g. "node 4" or, with an alias, "node 4 (dave)". */
export const elementLabel = (element: GraphElement): string =>
  `${isNode(element) ? "node" : "edge"} ${element.id}${element.alias ? ` (${element.alias})` : ""}`;

const plural = (count: number, word: string): string =>
  `${count} ${word}${count === 1 ? "" : "s"}`;

export type GraphActionsOptions = {
  /** Keep pins of elements missing from the results (a saved view, e.g. a dashboard widget). */
  keepMissingPins?: boolean;
};

export const createGraphActions = (
  queryId: Ref<string | undefined> | undefined,
  { keepMissingPins = false }: GraphActionsOptions = {},
) => {
  const queryStore = useQueryStore();
  const { exec, execMut } = useDbExec();
  const { openModal } = useModal();
  const { query, graphData, elementsById } = useGraphData(queryId);

  const selectedId = ref<number>();
  const panelMode = ref<PanelMode>("view");
  const menu = ref<ContextMenuState>();
  const pendingEdgeSourceId = ref<number>();
  /** Elements whose details stay shown on the graph, in pinning order (kept in the query). */
  const pins = computed(() => query.value?.pins ?? []);
  const pinnedIds = computed(() => pins.value.map((pin) => pin.id));
  const setPins = (next: PinnedDetail[]) => {
    if (queryId?.value) queryStore.setPins(queryId.value, next);
  };
  /** The edge created last, so the graph can animate it once. */
  const lastAddedEdgeId = ref<number>();

  const elementValues = useElementValues(exec);

  // Load stored values of shown elements so tooltips and the panel show all
  // data, not just what the query returned. A new run may have changed
  // values, so the cache starts over.
  let cachedForRun = query.value?.lastRun;
  let aliasesLoaded = false;
  watch(
    [() => query.value?.lastRun, elementsById],
    ([lastRun, byId]) => {
      const newRun = lastRun !== cachedForRun;
      if (newRun) {
        cachedForRun = lastRun;
        elementValues.clear();
      }
      void elementValues.load([...byId.keys()].slice(0, PREFETCH_LIMIT));
      // A run may have added or removed aliases; nothing to label without elements.
      if (newRun) aliasesLoaded = false;
      if (!aliasesLoaded && byId.size > 0) {
        aliasesLoaded = true;
        void elementValues.loadAliases();
      }
    },
    { immediate: true },
  );

  /** Loads stored values of one element (e.g. beyond the prefetch limit). */
  const loadValues = (id: number) => elementValues.load([id], { retry: true });

  const selectedElement = computed(() => {
    if (selectedId.value === undefined) return undefined;
    const element = elementsById.value.get(selectedId.value);
    return element && elementValues.describe(element);
  });

  const labelOf = (element: GraphElement): string =>
    elementLabel(elementValues.describe(element));

  /** Thrown when no database is open, so nothing could run. */
  const NO_DATABASE = "No database is open, so nothing was changed.";

  const notifyError = (title: string, error: unknown) => {
    addNotification({ type: "error", title, message: errorMessage(error) });
  };

  /** Incremented per refresh; only the latest refresh may store results. */
  let refreshToken = 0;

  /**
   * Re-runs the queries of the last read run (exactly as they were sent, not
   * the current builder rows) so the graph matches the database. A mutation
   * run (`exec_mut`) is never repeated — the caller patches the stored
   * results locally instead. A refresh never overwrites results of a run
   * that started or finished in the meantime.
   */
  const refresh = async (): Promise<void> => {
    const id = queryId?.value;
    const current = query.value;
    if (!id || !current || current.lastExecutedTab !== "exec") return;
    if (current.isRunning) return;

    const token = ++refreshToken;
    const startedAfterRun = current.lastRun;
    const isStale = () =>
      token !== refreshToken ||
      !!query.value?.isRunning ||
      query.value?.lastRun !== startedAfterRun;

    const execQueries = current.executedQueries?.exec ?? [];
    const contextQueries = current.executedQueries?.context ?? [];
    try {
      const [execResults, contextResults] = await Promise.all([
        execQueries.length ? exec(execQueries) : undefined,
        contextQueries.length ? exec(contextQueries) : undefined,
      ]);
      if (isStale()) return;
      if (execResults) {
        queryStore.setQueryResults(id, "exec", execResults, {
          keepExpanded: true,
        });
      }
      if (contextResults) {
        queryStore.setQueryResults(id, "context", contextResults);
      }
    } catch (error) {
      if (isStale()) return;
      // The change itself succeeded; the locally patched results are kept.
      addNotification({
        type: "warning",
        title: "Graph not refreshed",
        message: `Re-running the query failed: ${errorMessage(error)}`,
      });
    }
  };

  const closeMenu = () => {
    menu.value = undefined;
  };

  /** Whether the details form has edits that closing it would lose. */
  const unsavedEdits = ref(false);
  const setUnsavedEdits = (value: boolean) => {
    unsavedEdits.value = value;
  };

  /** Keeps unsaved edits open (with a hint) instead of discarding them. */
  const keepUnsavedEdits = (): boolean => {
    if (!unsavedEdits.value || panelMode.value !== "edit") return false;
    addNotification({
      type: "warning",
      title: "Unsaved changes",
      message: "Save or cancel the changes to the values first.",
    });
    return true;
  };

  /** Closes the panel, discarding edits (e.g. its close button). */
  const closePanel = () => {
    selectedId.value = undefined;
    panelMode.value = "view";
    unsavedEdits.value = false;
  };

  const cancelAddEdge = () => {
    pendingEdgeSourceId.value = undefined;
  };

  /** Closes everything (menu, panel, add-edge mode). */
  const dismiss = () => {
    closeMenu();
    cancelAddEdge();
    if (!keepUnsavedEdits()) closePanel();
  };

  // Colouring. Result `i` came from the `i`-th row the last run sent.
  const rowIdsByResult = computed(() => {
    const tab = query.value?.lastExecutedTab;
    if (!tab) return [];
    return query.value?.executedRowIds?.[tab] ?? [];
  });

  /** Number of colours in the palette; set by the graph from the theme. */
  const paletteSize = ref(GRAPH_THEME_FALLBACK.palette.length);

  /** A shown element by id, as labelled in menus and colours. */
  const labelOfId = (id: number): string => {
    const element = elementsById.value.get(id);
    return element
      ? elementLabel(elementValues.describe(element))
      : `element ${id}`;
  };

  /** The node the first search of the last run started from, if shown. */
  const searchStartId = computed(() => {
    const tab = query.value?.lastExecutedTab;
    const queries = tab ? (query.value?.executedQueries?.[tab] ?? []) : [];
    const id = searchStart(queries, (alias) => {
      for (const element of elementsById.value.values()) {
        if (isNode(element) && elementValues.describe(element).alias === alias)
          return element.id;
      }
      return undefined;
    });
    // A search can start from an edge, but distances are from nodes.
    const element = id === undefined ? undefined : elementsById.value.get(id);
    return element && isNode(element) ? id : undefined;
  });

  const colorResolver = computed(() =>
    buildColorResolver(
      query.value?.colors,
      [...elementsById.value.values()].map(elementValues.describe),
      rowIdsByResult.value,
      paletteSize.value,
      {
        // Only rows of the last run colour something, so are in the legend.
        row: (rowId) => `query ${rowIdsByResult.value.indexOf(rowId)}`,
        element: labelOfId,
        startId: searchStartId.value,
      },
    ),
  );

  /** The colour of an element, using all its stored values. */
  const colorOf = (element: GraphElement): ColorRef | undefined =>
    colorResolver.value.colorOf(elementValues.describe(element));
  const colorLegend = computed(() => colorResolver.value.legend);
  const colorRules = computed(() => query.value?.colors?.rules ?? []);
  const rowColors = computed(() => query.value?.colors?.rows ?? {});

  const setElementColor = (element: GraphElement, slot?: PaletteSlot) => {
    if (queryId?.value)
      queryStore.setElementColor(queryId.value, element.id, slot);
  };
  const setRowColor = (rowId: string, slot?: PaletteSlot) => {
    if (queryId?.value) queryStore.setRowColor(queryId.value, rowId, slot);
  };
  const setColorRules = (rules: ColorRule[]) => {
    if (queryId?.value) queryStore.setColorRules(queryId.value, rules);
  };
  const clearColors = () => {
    if (queryId?.value) queryStore.clearColors(queryId.value);
  };
  /**
   * Colours nodes by their distance from `from` (a node id, or the search
   * start), first among the rules. It replaces an earlier distance rule, so
   * one distance colouring is shown at a time, and keeps its scale.
   */
  const colorByDistance = (from: number | "start") => {
    const previous = colorRules.value.find((rule) => rule.kind === "distance");
    const others = colorRules.value.filter((rule) => rule.kind !== "distance");
    setColorRules([
      {
        kind: "distance",
        id: crypto.randomUUID(),
        from,
        scale: previous ? ruleScale(previous) : unusedScale(others),
      },
      ...others,
    ]);
  };
  /**
   * The keys of the shown elements with their values and how many elements
   * have each, offered for colour rules. Values are sorted by count.
   */
  const valueIndex = computed(() => {
    const index = new Map<string, Map<string, number>>();
    for (const element of elementsById.value.values()) {
      for (const v of elementValues.describe(element).values) {
        const counts = index.get(v.key) ?? new Map<string, number>();
        counts.set(v.value, (counts.get(v.value) ?? 0) + 1);
        index.set(v.key, counts);
      }
    }
    return [...index.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, counts]) => ({
        key,
        /** Every value reads as a number, so the key can be a gradient. */
        numeric: [...counts.keys()].every(
          (value) => numericValue(value) !== undefined,
        ),
        values: [...counts.entries()]
          .map(([value, count]) => ({ value, count }))
          .sort((a, b) => b.count - a.count || a.value.localeCompare(b.value)),
      }));
  });

  /**
   * The queries of the last run, for colouring by query. Labels come from the
   * rows as they are now; a row removed since the run has none.
   */
  const colorQueries = computed(() => {
    const tab = query.value?.lastExecutedTab;
    const rows = tab ? query.value?.rows[tab] : undefined;
    return rowIdsByResult.value.map((rowId, index) => {
      const row = rows?.find((r) => r.id === rowId);
      return {
        rowId,
        index,
        label: row
          ? row.steps.map((step) => getQueryStepLabel(step.type)).join(".")
          : "(removed)",
        slot: query.value?.colors?.rows[rowId],
      };
    });
  });

  /** Gives each query of the last run its own colour, as far as the palette goes. */
  const colorEachQuery = () => {
    colorQueries.value.forEach(({ rowId, index }) =>
      setRowColor(rowId, index < paletteSize.value ? index : undefined),
    );
  };

  /** Elements coloured by hand, shown or not, with a label. */
  const colorElements = computed(() =>
    Object.entries(query.value?.colors?.elements ?? {}).map(([key, slot]) => {
      const id = Number(key);
      const element = elementsById.value.get(id);
      return {
        id,
        slot,
        element,
        label: element
          ? elementLabel(elementValues.describe(element))
          : `${id < 0 ? "edge" : "node"} ${id} (not shown)`,
      };
    }),
  );
  /** Sets (or with no slot clears) the colour of an element, shown or not. */
  const setElementColorById = (id: number, slot?: PaletteSlot) => {
    if (queryId?.value) queryStore.setElementColor(queryId.value, id, slot);
  };

  const isPinned = (id: number): boolean => pinnedIds.value.includes(id);

  // A card pinned again starts next to its element: unpinning drops its offset.
  const unpin = (id: number) => {
    setPins(pins.value.filter((pin) => pin.id !== id));
  };

  const togglePin = (element: GraphElement) => {
    if (isPinned(element.id)) {
      unpin(element.id);
      return;
    }
    setPins([...pins.value, { id: element.id }]);
    void loadValues(element.id);
  };

  /** Where a pinned card sits relative to its element (px); unset is the default spot. */
  const cardOffset = (id: number): ScreenPoint | undefined =>
    pins.value.find((pin) => pin.id === id)?.offset;
  const setCardOffset = (id: number, offset: ScreenPoint) => {
    if (queryId?.value) queryStore.setPinOffset(queryId.value, id, offset);
  };

  const pinnedElements = computed(() =>
    pinnedIds.value.flatMap((id) => {
      const element = elementsById.value.get(id);
      return element ? [elementValues.describe(element)] : [];
    }),
  );

  // Pins (e.g. restored with a saved view) show stored values beyond the prefetch.
  watch(
    [pinnedIds, elementsById],
    ([ids, byId]) => {
      const shown = ids.filter((id) => byId.has(id));
      if (shown.length) void elementValues.load(shown);
    },
    { immediate: true },
  );

  // Pins of elements no longer shown (e.g. deleted, or a new run) are
  // dropped, unless they are kept with a saved view (they show again when
  // their element is back).
  watch(elementsById, (byId) => {
    if (keepMissingPins) return;
    const kept = pins.value.filter((pin) => byId.has(pin.id));
    if (kept.length !== pins.value.length) setPins(kept);
  });

  const openMenu = (element: GraphElement, x: number, y: number) => {
    menu.value = { element, x, y };
  };

  const viewElement = (element: GraphElement) => {
    if (keepUnsavedEdits()) return;
    selectedId.value = element.id;
    panelMode.value = "view";
    void loadValues(element.id);
  };

  const editElement = (element: GraphElement) => {
    if (selectedId.value !== element.id && keepUnsavedEdits()) return;
    selectedId.value = element.id;
    panelMode.value = "edit";
    void loadValues(element.id);
  };

  /** Leaves edit mode without saving; the panel stays open. */
  const cancelEdit = () => {
    panelMode.value = "view";
    unsavedEdits.value = false;
  };

  /**
   * Writes the edited values. `original` should be the values when editing
   * started: only differences to it are written, so values changed elsewhere
   * meanwhile are not overwritten by untouched rows.
   */
  const saveValues = async (
    element: GraphElement,
    edited: DbKeyValue[],
    original: DbKeyValue[] = element.rawValues,
  ): Promise<boolean> => {
    const id = queryId?.value;
    if (!id) return false;
    const queries = updateValuesQueries(element.id, original, edited);
    if (!queries.length) {
      addNotification({
        type: "info",
        title: "No changes",
        message: `The values of ${labelOf(element)} were not changed.`,
      });
      cancelEdit();
      return true;
    }
    try {
      if (!(await execMut(queries))) throw new Error(NO_DATABASE);
    } catch (error) {
      notifyError("Saving values failed", error);
      return false;
    }

    addNotification({
      type: "success",
      title: "Values saved",
      message: `Values of ${labelOf(element)} were updated.`,
    });
    cancelEdit();

    // The write succeeded; failing to reload only leaves the old values shown.
    try {
      const fresh = await exec([selectElementQuery(element.id)]);
      const updated = fresh?.[0]?.elements[0];
      if (updated) elementValues.set(element.id, updated.values);
    } catch (error) {
      addNotification({
        type: "warning",
        title: "Values not reloaded",
        message: `The values were saved, but reloading ${labelOf(element)} failed: ${errorMessage(error)}`,
      });
    }
    await refresh();
    return true;
  };

  const deleteElement = async (element: GraphElement): Promise<boolean> => {
    const id = queryId?.value;
    if (!id) return false;
    // Before the cache (and its alias) is cleared.
    const label = labelOf(element);
    try {
      if (!(await execMut([deleteElementQuery(element.id)]))) {
        throw new Error(NO_DATABASE);
      }
      queryStore.removeElementsFromResults(id, [element.id]);
      elementValues.remove(element.id);
      if (selectedId.value === element.id) closePanel();
      if (pendingEdgeSourceId.value === element.id) cancelAddEdge();
      // The database reuses ids: an edge created later with the same id is
      // new and should animate too.
      lastAddedEdgeId.value = undefined;
      await refresh();
      addNotification({
        type: "success",
        title: "Element deleted",
        message: `The ${label} was deleted.`,
      });
      return true;
    } catch (error) {
      notifyError("Delete failed", error);
      return false;
    }
  };

  const confirmDelete = (element: GraphElement) => {
    const lines = [`Are you sure you want to delete ${labelOf(element)}?`];
    if (isNode(element)) {
      lines.push("All edges of the node will be deleted as well.");
    }
    openModal({
      header: `Delete ${labelOf(element)}`,
      content: convertArrayOfStringsToContent(lines, {
        emphasizedWords: ["delete"],
      }),
      onConfirm: () => deleteElement(element),
    });
  };

  const expandNeighbours = async (node: GraphNode): Promise<void> => {
    const id = queryId?.value;
    if (!id) return;
    try {
      const results = await exec(neighboursQueries(node.id));
      if (!results) return;
      queryStore.appendExpandedResults(id, results);
      const ids = new Set(
        results.flatMap((r) => r.elements.map((el) => el.id)),
      );
      // The expanded node itself is part of the results; don't count it.
      ids.delete(node.id);
      const nodes = [...ids].filter((elementId) => elementId > 0).length;
      const edges = ids.size - nodes;
      addNotification({
        type: "info",
        title: "Neighbours loaded",
        message: `Loaded ${plural(nodes, "node")} and ${plural(edges, "edge")} around node ${node.id}.`,
      });
    } catch (error) {
      notifyError("Loading neighbours failed", error);
    }
  };

  const startAddEdge = (node: GraphNode) => {
    pendingEdgeSourceId.value = node.id;
  };

  const addEdge = async (from: number, to: number): Promise<boolean> => {
    const id = queryId?.value;
    if (!id) return false;
    try {
      const results = await execMut([insertEdgeQuery(from, to)]);
      if (!results) throw new Error(NO_DATABASE);
      cancelAddEdge();
      queryStore.appendExpandedResults(id, results);
      await refresh();
      const edgeId = results[0]?.elements[0]?.id;
      addNotification({
        type: "success",
        title: "Edge created",
        message: `Edge ${edgeId ?? ""} from node ${from} to node ${to} was created.`,
      });
      lastAddedEdgeId.value = edgeId;
      // Let the user add values to the new edge right away.
      if (edgeId !== undefined) {
        selectedId.value = edgeId;
        panelMode.value = "edit";
      }
      return true;
    } catch (error) {
      notifyError("Creating edge failed", error);
      return false;
    }
  };

  /** Completes the add-edge mode with the chosen target node. */
  const completeAddEdge = (target: GraphNode) => {
    const from = pendingEdgeSourceId.value;
    if (from === undefined) return;
    openModal({
      header: "Add edge",
      content: convertArrayOfStringsToContent([
        `Create an edge from node ${from} to node ${target.id}?`,
        "You can add values to the edge after it is created.",
      ]),
      onConfirm: () => addEdge(from, target.id),
    });
  };

  /**
   * Handles a (left) click on a node: picks the edge target, pins it with
   * Shift, or opens its details.
   */
  const clickNode = (node: GraphNode, { pin = false } = {}) => {
    closeMenu();
    if (pendingEdgeSourceId.value !== undefined) {
      completeAddEdge(node);
      return;
    }
    if (pin) {
      togglePin(node);
      return;
    }
    viewElement(node);
  };

  const getActions = (element: GraphElement): Action<undefined>[] => {
    const run =
      (fn: () => unknown) =>
      ({ event }: { event: Event }) => {
        event.preventDefault();
        closeMenu();
        fn();
        return true;
      };
    const actions: Action<undefined>[] = [
      {
        key: "view",
        label: "View details",
        action: run(() => viewElement(element)),
      },
      {
        key: "edit",
        label: "Edit values",
        action: run(() => editElement(element)),
      },
      {
        key: "pin",
        label: isPinned(element.id) ? "Unpin details" : "Pin details",
        action: run(() => togglePin(element)),
      },
    ];
    if (isNode(element)) {
      actions.push(
        {
          key: "expand",
          label: "Expand neighbours",
          action: run(() => expandNeighbours(element)),
        },
        {
          key: "add-edge",
          label: "Add edge from here",
          action: run(() => startAddEdge(element)),
        },
        {
          key: "color-distance",
          label: "Colour by distance from here",
          action: run(() => colorByDistance(element.id)),
        },
      );
    }
    actions.push({
      key: "delete",
      label: "Delete",
      action: run(() => confirmDelete(element)),
    });
    return actions;
  };

  return {
    graphData,
    elementsById,
    selectedId,
    selectedElement,
    describeElement: elementValues.describe,
    aliasOf: elementValues.aliasOf,
    loadValues,
    panelMode: readonly(panelMode),
    menu,
    pendingEdgeSourceId,
    pinnedIds,
    pinnedElements,
    cardOffset,
    setCardOffset,
    lastAddedEdgeId: readonly(lastAddedEdgeId),
    isPinned,
    togglePin,
    paletteSize,
    colorResolver,
    colorOf,
    colorLegend,
    labelOfId,
    searchStartId,
    colorByDistance,
    colorRules,
    rowColors,
    valueIndex,
    colorQueries,
    colorEachQuery,
    colorElements,
    setElementColorById,
    setElementColor,
    setRowColor,
    setColorRules,
    clearColors,
    unpin,
    openMenu,
    closeMenu,
    closePanel,
    setUnsavedEdits,
    cancelAddEdge,
    dismiss,
    viewElement,
    editElement,
    cancelEdit,
    saveValues,
    deleteElement,
    confirmDelete,
    expandNeighbours,
    startAddEdge,
    addEdge,
    completeAddEdge,
    clickNode,
    getActions,
    refresh,
  };
};

export type GraphActions = ReturnType<typeof createGraphActions>;

export const GRAPH_ACTIONS_KEY: InjectionKey<GraphActions> =
  Symbol("graphActions");

/** Creates graph actions for a query and provides them to descendants. */
export const provideGraphActions = (
  queryId: Ref<string | undefined> | undefined,
  options?: GraphActionsOptions,
): GraphActions => {
  const actions = createGraphActions(queryId, options);
  provide(GRAPH_ACTIONS_KEY, actions);
  return actions;
};

export const useGraphActions = (): GraphActions | undefined =>
  inject(GRAPH_ACTIONS_KEY, undefined);
