import type { Components } from "@agnesoft/agdb_api";
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
import type { GraphElement, GraphNode } from "./types";
import {
  deleteElementQuery,
  insertEdgeQuery,
  neighboursQueries,
  selectElementQuery,
  updateValuesQueries,
} from "../utils/graphActionQueries";

export type PanelMode = "view" | "edit";

export type ContextMenuState = {
  x: number;
  y: number;
  element: GraphElement;
};

export const isNode = (element: GraphElement): element is GraphNode =>
  element.id > 0;

export const elementLabel = (element: GraphElement): string =>
  `${isNode(element) ? "node" : "edge"} ${element.id}`;

const plural = (count: number, word: string): string =>
  `${count} ${word}${count === 1 ? "" : "s"}`;

const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const createGraphActions = (
  queryId: Ref<string | undefined> | undefined,
) => {
  const queryStore = useQueryStore();
  const { exec, execMut } = useDbExec();
  const { openModal } = useModal();
  const { query, graphData, elementsById } = useGraphData(queryId);

  const selectedId = ref<number>();
  const panelMode = ref<PanelMode>("view");
  const menu = ref<ContextMenuState>();
  const pendingEdgeSourceId = ref<number>();

  const elementValues = useElementValues(exec);

  // Load stored values of shown elements so tooltips and the panel show all
  // data, not just what the query returned. A new run may have changed
  // values, so the cache starts over.
  let cachedForRun = query.value?.lastRun;
  watch(
    [() => query.value?.lastRun, elementsById],
    ([lastRun, byId]) => {
      if (lastRun !== cachedForRun) {
        cachedForRun = lastRun;
        elementValues.clear();
      }
      void elementValues.load([...byId.keys()].slice(0, PREFETCH_LIMIT));
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
    edited: Components.Schemas.DbKeyValue[],
    original: Components.Schemas.DbKeyValue[] = element.rawValues,
  ): Promise<boolean> => {
    const id = queryId?.value;
    if (!id) return false;
    const queries = updateValuesQueries(element.id, original, edited);
    if (!queries.length) {
      addNotification({
        type: "info",
        title: "No changes",
        message: `The values of ${elementLabel(element)} were not changed.`,
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
      message: `Values of ${elementLabel(element)} were updated.`,
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
        message: `The values were saved, but reloading ${elementLabel(element)} failed: ${errorMessage(error)}`,
      });
    }
    await refresh();
    return true;
  };

  const deleteElement = async (element: GraphElement): Promise<boolean> => {
    const id = queryId?.value;
    if (!id) return false;
    try {
      if (!(await execMut([deleteElementQuery(element.id)]))) {
        throw new Error(NO_DATABASE);
      }
      queryStore.removeElementsFromResults(id, [element.id]);
      elementValues.remove(element.id);
      if (selectedId.value === element.id) closePanel();
      if (pendingEdgeSourceId.value === element.id) cancelAddEdge();
      await refresh();
      addNotification({
        type: "success",
        title: "Element deleted",
        message: `The ${elementLabel(element)} was deleted.`,
      });
      return true;
    } catch (error) {
      notifyError("Delete failed", error);
      return false;
    }
  };

  const confirmDelete = (element: GraphElement) => {
    const lines = [`Are you sure you want to delete ${elementLabel(element)}?`];
    if (isNode(element)) {
      lines.push("All edges of the node will be deleted as well.");
    }
    openModal({
      header: `Delete ${elementLabel(element)}`,
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

  /** Handles a (left) click on a node: picks the edge target or opens details. */
  const clickNode = (node: GraphNode) => {
    closeMenu();
    if (pendingEdgeSourceId.value !== undefined) {
      completeAddEdge(node);
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
    loadValues,
    panelMode: readonly(panelMode),
    menu,
    pendingEdgeSourceId,
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
): GraphActions => {
  const actions = createGraphActions(queryId);
  provide(GRAPH_ACTIONS_KEY, actions);
  return actions;
};

export const useGraphActions = (): GraphActions | undefined =>
  inject(GRAPH_ACTIONS_KEY, undefined);
