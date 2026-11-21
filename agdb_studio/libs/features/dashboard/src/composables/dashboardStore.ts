import { ref } from "vue";
import { checkClient, client } from "@agdb-studio/api/src/api";
import type { DashboardSummary } from "@agnesoft/agdb_api";
import type { DashboardDocument, Widget, WidgetSize } from "../types";
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
/** The open dashboard has edits that are not saved. */
const dirty = ref(false);
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

/** Saves the open dashboard. */
const saveDashboard = async (): Promise<void> => {
  if (!current.value) return;
  const sent = edits;
  await saveDocument(current.value.name, current.value.document);
  // Edits made while the request ran are not in it.
  if (edits === sent) dirty.value = false;
};

const renameDashboard = async (name: string, newName: string) => {
  // Keeping the name is not a conflict with itself.
  if (newName === name) return;
  checkClient(client);
  await client.value.dashboard_rename({ name, new_name: newName });
  if (current.value?.name === name) current.value.name = newName;
  await fetchDashboards();
};

const removeDashboard = async (name: string) => {
  checkClient(client);
  await client.value.dashboard_remove({ name });
  if (current.value?.name === name) {
    current.value = undefined;
    dirty.value = false;
  }
  await fetchDashboards();
};

/**
 * Appends widgets to a saved dashboard in one save, e.g. when pinning queries
 * from the query page (the dashboard page is closed then).
 */
const appendWidgets = async (name: string, added: Widget[]): Promise<void> => {
  checkClient(client);
  const response = await client.value.dashboard_get({ name });
  const document = parseDocument(response.data.config);
  addToDocument(document, added);
  await saveDocument(name, document);
};

/** Runs an edit of the open dashboard; edits that change something mark it unsaved. */
const edit = (change: (doc: DashboardDocument) => boolean) => {
  if (current.value && change(current.value.document)) {
    edits++;
    dirty.value = true;
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

const removeWidget = (id: string) =>
  edit((doc) => {
    removeFromDocument(doc, id);
    return true;
  });

/** Moves a widget by `delta` places (-1 = earlier). Moves past either end are ignored. */
const moveWidget = (id: string, delta: number) =>
  edit((doc) => moveInDocument(doc, id, delta));

const resizeWidget = (id: string, size: WidgetSize) =>
  edit((doc) => resizeInDocument(doc, id, size));

/** Closes the open dashboard, dropping unsaved edits. */
const closeDashboard = () => {
  current.value = undefined;
  dirty.value = false;
};

export const useDashboardStore = () => ({
  dashboards,
  current,
  dirty,
  fetchDashboards,
  loadDashboard,
  createDashboard,
  saveDashboard,
  renameDashboard,
  removeDashboard,
  appendWidgets,
  addWidget,
  updateWidget,
  removeWidget,
  moveWidget,
  resizeWidget,
  closeDashboard,
});
