import { onScopeDispose, watch } from "vue";
import { onBeforeRouteLeave, useRoute, useRouter } from "vue-router";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { useQueryStore } from "./queryStore";
import { TABS } from "./types";
import {
  MAX_QUERY_URL_LENGTH,
  rowsToUrlParams,
  sameUrlParams,
  urlParamsLength,
  urlParamsToRows,
  type QueryUrlParams,
} from "../utils/queryUrl";

/** Wait after the last edit before updating the URL, e.g. while typing. */
export const URL_UPDATE_DELAY = 300;

const useOptionalRouter = () => {
  try {
    // Both return undefined (with a warning) outside of a router.
    const router = useRouter();
    const route = useRoute();
    return router && route ? { router, route } : undefined;
  } catch {
    /* v8 ignore next -- @preserve: only when the injection itself throws */
    return undefined;
  }
};

/**
 * Keeps the builder rows of a query in the URL (`?exec=…&context=…`, one
 * value per row) so a view can be saved and shared. Call during setup; does
 * nothing without a router.
 *
 * Rows longer than `MAX_QUERY_URL_LENGTH` are not kept in the URL (with a
 * warning), and links longer than that are not opened.
 */
export const useQueryUrlSync = () => {
  const routing = useOptionalRouter();
  const queryStore = useQueryStore();
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopWatch: (() => void) | undefined;
  // Warn once each time the rows grow too long, not on every edit.
  let tooLongWarned = false;

  const stop = () => {
    clearTimeout(timer);
    stopWatch?.();
  };
  onScopeDispose(stop);
  // A URL update after leaving would start a navigation back to this page
  // and cancel the one the user started (guards are async).
  if (routing) onBeforeRouteLeave(stop);

  const updateUrl = (queryId: string) => {
    const query = queryStore.getQuery(queryId);
    if (!routing || !query) return;
    const { router, route } = routing;
    let params: QueryUrlParams = rowsToUrlParams(query.value.rows);
    if (urlParamsLength(params) > MAX_QUERY_URL_LENGTH) {
      // A stale link would be misleading, so the rows are dropped from it.
      params = {};
      if (!tooLongWarned) {
        tooLongWarned = true;
        addNotification({
          type: "warning",
          title: "Queries too long for the link",
          message:
            "The link no longer holds the queries. Copy the rows to share them instead.",
        });
      }
    } else {
      tooLongWarned = false;
    }
    if (sameUrlParams(route.query, params)) return;
    const next = { ...route.query };
    for (const tab of TABS) {
      const values = params[tab];
      if (values) next[tab] = values;
      else delete next[tab];
    }
    void router.replace({ query: next });
  };

  const restore = (queryId: string, query: Record<string, unknown>) => {
    if (urlParamsLength(query) > MAX_QUERY_URL_LENGTH) {
      addNotification({
        type: "warning",
        title: "Link too long",
        message: "The queries in the link are too long to open.",
      });
      return;
    }
    const { rows, skipped } = urlParamsToRows(query);
    // Mutations run as one transaction, so running only some of them could
    // do something other than intended: none are opened then.
    if (skipped.exec_mut) delete rows.exec_mut;
    for (const tab of TABS) {
      const tabRows = rows[tab];
      if (tabRows) queryStore.setQueryRows(queryId, tab, tabRows);
    }
    // Mutations from a link could be harmful, so running them asks first.
    if (rows.exec_mut) queryStore.setUnconfirmedLinkedMutation(queryId, true);
    const count = TABS.reduce((sum, tab) => sum + (skipped[tab] ?? 0), 0);
    if (count) {
      addNotification({
        type: "warning",
        title: "Some queries were not opened",
        message: `${count} ${count === 1 ? "query" : "queries"} in the link could not be read.${
          skipped.exec_mut ? " The exec_mut queries were not opened." : ""
        }`,
      });
    }
  };

  /** Restores the rows from the URL, then keeps the URL up to date. */
  const start = (queryId: string) => {
    const query = queryStore.getQuery(queryId);
    if (!routing || !query) return;
    restore(queryId, routing.route.query);
    stopWatch = watch(
      () => query.value.rows,
      () => {
        clearTimeout(timer);
        timer = setTimeout(() => updateUrl(queryId), URL_UPDATE_DELAY);
      },
      { deep: true },
    );
  };

  return { start, stop };
};
