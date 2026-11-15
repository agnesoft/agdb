import { beforeEach, describe, expect, it, vi } from "vitest";

const { addNotification } = vi.hoisted(() => ({ addNotification: vi.fn() }));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import { notifyCopyResult, notifyPasteFailure } from "./rowCopyNotification";

describe("rowCopyNotification", () => {
  beforeEach(() => {
    addNotification.mockClear();
  });

  describe("notifyCopyResult", () => {
    it("names one query or the number of queries", () => {
      notifyCopyResult("copied");
      notifyCopyResult("copied", 3);
      expect(addNotification).toHaveBeenNthCalledWith(1, {
        type: "success",
        title: "Query copied",
      });
      expect(addNotification).toHaveBeenNthCalledWith(2, {
        type: "success",
        title: "3 queries copied",
      });
    });

    it("explains a copy kept in Studio only", () => {
      notifyCopyResult("local", 2);
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "info",
          title: "2 queries copied in Studio only",
        }),
      );
    });

    it("reports a failed copy", () => {
      notifyCopyResult("failed");
      expect(addNotification).toHaveBeenCalledWith({
        type: "error",
        title: "Copy to clipboard failed",
      });
    });
  });

  describe("notifyPasteFailure", () => {
    it("suggests the keyboard when clipboard access is denied", () => {
      notifyPasteFailure({ denied: true });
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "warning",
          title: "Clipboard access denied",
        }),
      );
    });

    it("shows why the text could not be pasted", () => {
      notifyPasteFailure({ error: "Unknown step" });
      expect(addNotification).toHaveBeenCalledWith({
        type: "warning",
        title: "Could not paste the query",
        message: "Unknown step",
      });
    });
  });
});
