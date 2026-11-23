import { computed, ref } from "vue";
import { checkClient, client } from "@agdb-studio/api/src/api";
import type { DashboardSummary, DbUserRole } from "@agnesoft/agdb_api";
import { useAccount } from "@agdb-studio/auth/src/account";
import { useAdmin } from "@agdb-studio/router/src/admin";
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

/** Dashboards are unique per owner, so both identify one. */
export type DashboardKey = { owner: string; name: string };

export type OpenDashboard = DashboardKey & {
  /** The current user's role on the dashboard. */
  role: DbUserRole;
  /** Every user can view a public dashboard. */
  public: boolean;
  document: DashboardDocument;
};

const { username } = useAccount();
const { isAdmin, isAdminView } = useAdmin();

/**
 * In the admin view the server admin manages every user's dashboards through
 * the admin routes, as the owner would.
 */
export const adminMode = (): boolean => isAdminView.value && isAdmin.value;

/** The route of a dashboard's page (the admin one in the admin view). */
export const dashboardRoute = (key: DashboardKey) => ({
  name: adminMode() ? "admin-dashboard" : "dashboard",
  params: { owner: key.owner, name: key.name },
});

const dashboards = ref<DashboardSummary[]>([]);
/** Dashboards the current user owns. */
const mine = computed(() =>
  dashboards.value.filter((d) => d.owner === username.value),
);
/** Dashboards others shared with the current user (public or not). */
const shared = computed(() =>
  dashboards.value.filter((d) => d.owner !== username.value && d.member),
);
/** Public dashboards the current user has no role on. */
const publicOnly = computed(() =>
  dashboards.value.filter((d) => d.owner !== username.value && !d.member),
);
const current = ref<OpenDashboard>();
/** The dashboard the user opens at the start page, if any. */
const home = ref<DashboardKey>();
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

export const sameDashboard = (a?: DashboardKey, b?: DashboardKey): boolean =>
  !!a && !!b && a.owner === b.owner && a.name === b.name;

/** Write and admin may change the widgets, as `exec_mut` on databases. */
export const canEdit = (role: DbUserRole): boolean => role !== "read";

/** Only admins share the dashboard with others. */
export const canShare = (role: DbUserRole): boolean => role === "admin";

/**
 * Only the owner renames, deletes or publishes the dashboard, as with
 * databases; in the admin view the server admin acts as every owner.
 */
export const isOwner = (key: DashboardKey): boolean =>
  key.owner === username.value || adminMode();

/** "name" for own dashboards, "owner/name" for others (and in the admin view). */
export const dashboardLabel = (key: DashboardKey): string =>
  key.owner === username.value && !adminMode()
    ? key.name
    : `${key.owner}/${key.name}`;

const fetchDashboards = async (): Promise<void> => {
  checkClient(client);
  const response = adminMode()
    ? await client.value.admin_dashboard_list()
    : await client.value.dashboard_list();
  dashboards.value = response[1];
};

const getDashboard = async (key: DashboardKey) => {
  checkClient(client);
  const params = { owner: key.owner, name: key.name };
  return adminMode()
    ? await client.value.admin_dashboard_get(params)
    : await client.value.dashboard_get(params);
};

/**
 * Opens a dashboard. The previous one closes at once, so edits never go to a
 * dashboard other than the one being shown, even if this load fails.
 */
const loadDashboard = async (key: DashboardKey): Promise<OpenDashboard> => {
  checkClient(client);
  const id = ++loadId;
  current.value = undefined;
  dirty.value = false;
  saveFailed.value = false;
  await settleSaves();
  const response = await getDashboard(key);
  const open = {
    owner: key.owner,
    name: key.name,
    role: response[1].role,
    public: response[1].public,
    document: parseDocument(response[1].config),
  };
  if (id === loadId) current.value = open;
  return open;
};

/**
 * Creates a dashboard owned by the current user or, by the server admin in
 * the admin view, by `owner`.
 */
const createDashboard = async (
  name: string,
  owner?: string,
): Promise<DashboardKey> => {
  checkClient(client);
  const forOwner = adminMode() && owner !== undefined;
  const key = { owner: forOwner ? owner : (username.value ?? ""), name };
  const body = { config: serializeDocument(emptyDocument()) };
  if (forOwner) await client.value.admin_dashboard_add({ ...key, ...body });
  else await client.value.dashboard_add({ ...key, ...body });
  await fetchDashboards();
  return key;
};

const saveDocument = async (
  key: DashboardKey,
  document: DashboardDocument,
): Promise<void> => {
  checkClient(client);
  const params = { owner: key.owner, name: key.name };
  const body = { config: serializeDocument(document) };
  if (adminMode())
    await client.value.admin_dashboard_save({ ...params, ...body });
  else await client.value.dashboard_save({ ...params, ...body });
};

/** Dashboards to save (again) after the running save. */
const toSave = new Set<OpenDashboard>();
let saveLoop: Promise<void> | undefined;

/**
 * Waits for running saves, so a read gets what they wrote. Their failures
 * are already shown (`saveFailed`, the API error notification).
 */
const settleSaves = async (): Promise<void> => {
  // A loop may start again for edits made while the last one was finishing.
  while (saveLoop) await saveLoop;
};

const runSaves = async (): Promise<void> => {
  saving.value = true;
  try {
    for (const open of toSave) {
      toSave.delete(open);
      const sent = edits;
      try {
        await saveDocument(open, open.document);
      } catch {
        // The studio client shows a notification; edits are kept.
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
  return startSaves();
};

/** Runs the queued saves unless they run already. */
const startSaves = (): Promise<void> =>
  (saveLoop ??= runSaves().finally(() => {
    saveLoop = undefined;
    // Queued after the loop had seen the queue empty, but before it ended.
    if (toSave.size) void startSaves();
  }));

/** Closes the open dashboard, dropping unsaved edits (a running save still finishes). */
const closeDashboard = () => {
  current.value = undefined;
  dirty.value = false;
  saveFailed.value = false;
};

const renameDashboard = async (key: DashboardKey, newName: string) => {
  // Keeping the name is not a conflict with itself.
  if (newName === key.name) return;
  checkClient(client);
  // A save landing after the rename would bring the old name back.
  await settleSaves();
  const params = { owner: key.owner, name: key.name, new_name: newName };
  if (adminMode()) await client.value.admin_dashboard_rename(params);
  else await client.value.dashboard_rename(params);
  if (current.value && sameDashboard(current.value, key))
    current.value.name = newName;
  if (sameDashboard(home.value, key)) home.value = { ...key, name: newName };
  await fetchDashboards();
};

/** Forgets a dashboard that is gone or no longer accessible. */
const forget = (key: DashboardKey) => {
  if (sameDashboard(current.value, key)) closeDashboard();
  // The server drops the home with the dashboard or the access to it.
  if (sameDashboard(home.value, key)) home.value = undefined;
};

const removeDashboard = async (key: DashboardKey) => {
  checkClient(client);
  // A save landing after the removal would create the dashboard again.
  for (const open of toSave) if (sameDashboard(open, key)) toSave.delete(open);
  await settleSaves();
  const params = { owner: key.owner, name: key.name };
  if (adminMode()) await client.value.admin_dashboard_remove(params);
  else await client.value.dashboard_remove(params);
  forget(key);
  await fetchDashboards();
};

/** Makes a dashboard public (viewable by every user) or private. */
const setPublic = async (key: DashboardKey, isPublic: boolean) => {
  checkClient(client);
  const params = { owner: key.owner, name: key.name, is_public: isPublic };
  if (adminMode()) await client.value.admin_dashboard_public(params);
  else await client.value.dashboard_public(params);
  if (current.value && sameDashboard(current.value, key))
    current.value.public = isPublic;
  // Making it private drops it as the home of users without a role on it.
  await fetchDashboards();
};

/** Removes the current user's access to a dashboard shared with them. */
const leaveDashboard = async (key: DashboardKey) => {
  checkClient(client);
  // Saves after leaving would be rejected.
  for (const open of toSave) if (sameDashboard(open, key)) toSave.delete(open);
  await settleSaves();
  await client.value.dashboard_user_remove({
    ...key,
    username: username.value ?? "",
  });
  if (current.value && sameDashboard(current.value, key)) closeDashboard();
  // A public dashboard stays readable, and so stays the home on the server.
  await Promise.all([fetchDashboards(), fetchHome()]);
};

const fetchHome = async (): Promise<DashboardKey | undefined> => {
  checkClient(client);
  const response = await client.value.user_home_dashboard();
  home.value = response[1]
    ? { owner: response[1].owner, name: response[1].name }
    : undefined;
  return home.value;
};

/** Sets the dashboard opened at the start page; `undefined` clears it. */
const setHome = async (key?: DashboardKey): Promise<void> => {
  checkClient(client);
  if (key) {
    await client.value.user_home_dashboard_set({ ...key });
    home.value = { owner: key.owner, name: key.name };
  } else {
    await client.value.user_home_dashboard_clear();
    home.value = undefined;
  }
};

/**
 * Appends widgets to a saved dashboard in one save, e.g. when pinning queries
 * from the query page (the dashboard page is closed then).
 */
const appendWidgets = async (
  key: DashboardKey,
  added: Widget[],
): Promise<void> => {
  await settleSaves();
  const response = await getDashboard(key);
  const document = parseDocument(response[1].config);
  addToDocument(document, added);
  await saveDocument(key, document);
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
  mine,
  shared,
  publicOnly,
  current,
  home,
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
  setPublic,
  leaveDashboard,
  fetchHome,
  setHome,
  appendWidgets,
  addWidget,
  updateWidget,
  removeWidget,
  restoreWidget,
  moveWidget,
  resizeWidget,
  closeDashboard,
});
