import { ref } from "vue";
import { checkClient, client } from "@agdb-studio/api/src/api";
import type { DashboardSummary } from "@agnesoft/agdb_api";
import type {
  DashboardDocument,
  Widget,
  WidgetLayout,
  WidgetSize,
} from "../types";
import {
  addToDocument,
  emptyDocument,
  moveInDocument,
  parseDocument,
  removeFromDocument,
  resizeInDocument,
  serializeDocument,
} from "../utils/dashboardDocument";

export type OpenDashboard = {
  name: string;
  document: DashboardDocument;
};

const dashboards = ref<DashboardSummary[]>([]);
const current = ref<OpenDashboard>();
/** The open dashboard has edits that are not saved (yet). */
const dirty = ref(false);
/** A save of the open dashboard is running. */
const saving = ref(false);
/** The last save of the open dashboard failed; its edits are kept until a retry. */
const saveFailed = ref(false);
/** Counts edits, so a save only marks the edits it sent as saved. */
let edits = 0;
/** The latest load; responses of earlier loads are dropped. */
let loadId = 0;

const fetchDashboards = async (): Promise<void> => {
  checkClient(client);
  const response = await client.value.dashboard_list();
  dashboards.value = response.data;
};

/**
 * Opens a dashboard. The previous one closes at once, so edits never go to a
 * dashboard other than the one being shown, even if this load fails.
 */
const loadDashboard = async (name: string): Promise<OpenDashboard> => {
  checkClient(client);
  const id = ++loadId;
  current.value = undefined;
  dirty.value = false;
  saveFailed.value = false;
  await settleSaves();
  const response = await client.value.dashboard_get({ name });
  const open = { name, document: parseDocument(response.data.config) };
  if (id === loadId) current.value = open;
  return open;
};

const createDashboard = async (name: string): Promise<void> => {
  checkClient(client);
  await client.value.dashboard_add(
    { name },
    { config: serializeDocument(emptyDocument()) },
  );
  await fetchDashboards();
};

const saveDocument = async (
  name: string,
  document: DashboardDocument,
): Promise<void> => {
  checkClient(client);
  await client.value.dashboard_save(
    { name },
    { config: serializeDocument(document) },
  );
};

/** Dashboards to save (again) after the running save. */
const toSave = new Set<OpenDashboard>();
let saveLoop: Promise<void> | undefined;

/**
 * Waits for running saves, so a read gets what they wrote. Their failures
 * are already shown (`saveFailed`, the API error notification).
 */
const settleSaves = async (): Promise<void> => {
  await saveLoop;
};

const runSaves = async (): Promise<void> => {
  saving.value = true;
  try {
    for (const open of toSave) {
      toSave.delete(open);
      const sent = edits;
      try {
        await saveDocument(open.name, open.document);
      } catch {
        // The API error interceptor shows a notification; edits are kept.
        if (current.value === open) saveFailed.value = true;
        continue;
      }
      // Edits made while the request ran are not in it.
      if (current.value === open && edits === sent) {
        dirty.value = false;
        saveFailed.value = false;
      }
    }
  } finally {
    saving.value = false;
  }
};

/**
 * Saves the open dashboard. One save runs at a time; edits made meanwhile go
 * in one more save after it. Failures set `saveFailed` instead of throwing.
 */
const saveDashboard = (): Promise<void> => {
  if (!current.value) return Promise.resolve();
  toSave.add(current.value);
  saveLoop ??= runSaves().finally(() => (saveLoop = undefined));
  return saveLoop;
};

/** Closes the open dashboard, dropping unsaved edits (a running save still finishes). */
const closeDashboard = () => {
  current.value = undefined;
  dirty.value = false;
  saveFailed.value = false;
};

const renameDashboard = async (name: string, newName: string) => {
  // Keeping the name is not a conflict with itself.
  if (newName === name) return;
  checkClient(client);
  // A save landing after the rename would bring the old name back.
  await settleSaves();
  await client.value.dashboard_rename({ name, new_name: newName });
  if (current.value?.name === name) current.value.name = newName;
  await fetchDashboards();
};

const removeDashboard = async (name: string) => {
  checkClient(client);
  // A save landing after the removal would create the dashboard again.
  for (const open of toSave) if (open.name === name) toSave.delete(open);
  await settleSaves();
  await client.value.dashboard_remove({ name });
  if (current.value?.name === name) closeDashboard();
  await fetchDashboards();
};

/**
 * Appends widgets to a saved dashboard in one save, e.g. when pinning queries
 * from the query page (the dashboard page is closed then).
 */
const appendWidgets = async (name: string, added: Widget[]): Promise<void> => {
  checkClient(client);
  await settleSaves();
  const response = await client.value.dashboard_get({ name });
  const document = parseDocument(response.data.config);
  addToDocument(document, added);
  await saveDocument(name, document);
};

/** Runs an edit of the open dashboard; edits that change something are saved at once. */
const edit = (change: (doc: DashboardDocument) => boolean) => {
  if (current.value && change(current.value.document)) {
    edits++;
    dirty.value = true;
    void saveDashboard();
  }
};

const addWidget = (widget: Widget) =>
  edit((doc) => {
    addToDocument(doc, [widget]);
    return true;
  });

/** Replaces a widget's content; its place and size on the grid stay. */
const updateWidget = (widget: Widget) =>
  edit((doc) => {
    if (!Object.hasOwn(doc.widgets, widget.id)) return false;
    doc.widgets[widget.id] = widget;
    return true;
  });

/** A removed widget with its former place, so the removal can be undone. */
export type RemovedWidget = {
  widget: Widget;
  layout: WidgetLayout;
  index: number;
};

const removeWidget = (id: string): RemovedWidget | undefined => {
  const doc = current.value?.document;
  if (!doc || !Object.hasOwn(doc.widgets, id)) return undefined;
  const index = doc.layout.findIndex((entry) => entry.id === id);
  const layout = doc.layout[index];
  /* v8 ignore next -- @preserve: every widget has a layout entry */
  if (!layout) return undefined;
  const removed = { widget: doc.widgets[id]!, layout: { ...layout }, index };
  edit((d) => {
    removeFromDocument(d, id);
    return true;
  });
  return removed;
};

/** Puts a removed widget back where it was. */
const restoreWidget = ({ widget, layout, index }: RemovedWidget) =>
  edit((doc) => {
    if (Object.hasOwn(doc.widgets, widget.id)) return false;
    doc.widgets[widget.id] = widget;
    doc.layout.splice(Math.min(index, doc.layout.length), 0, layout);
    return true;
  });

/** Moves a widget by `delta` places (-1 = earlier). Moves past either end are ignored. */
const moveWidget = (id: string, delta: number) =>
  edit((doc) => moveInDocument(doc, id, delta));

const resizeWidget = (id: string, size: WidgetSize) =>
  edit((doc) => resizeInDocument(doc, id, size));

export const useDashboardStore = () => ({
  dashboards,
  current,
  dirty,
  saving,
  saveFailed,
  fetchDashboards,
  loadDashboard,
  createDashboard,
  saveDashboard,
  settleSaves,
  renameDashboard,
  removeDashboard,
  appendWidgets,
  addWidget,
  updateWidget,
  removeWidget,
  restoreWidget,
  moveWidget,
  resizeWidget,
  closeDashboard,
});
