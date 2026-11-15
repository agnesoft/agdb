import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import type { ReadResult } from "./queryRowClipboard";

/** Tells the user how copying one or more queries went. */
export const notifyCopyResult = (
  result: "copied" | "local" | "failed",
  count = 1,
): void => {
  const what = count === 1 ? "Query" : `${count} queries`;
  if (result === "copied") {
    addNotification({ type: "success", title: `${what} copied` });
  } else if (result === "local") {
    addNotification({
      type: "info",
      title: `${what} copied in Studio only`,
      message:
        "The system clipboard is not available here; paste it with the paste button.",
    });
  } else {
    addNotification({ type: "error", title: "Copy to clipboard failed" });
  }
};

/** Warns when reading the clipboard did not give any rows. */
export const notifyPasteFailure = (
  read: Exclude<ReadResult, { rows: unknown }>,
): void => {
  if ("denied" in read) {
    addNotification({
      type: "warning",
      title: "Clipboard access denied",
      message: "Click into the query and press Ctrl+V (Cmd+V on Mac) instead.",
    });
  } else {
    addNotification({
      type: "warning",
      title: "Could not paste the query",
      message: read.error,
    });
  }
};
