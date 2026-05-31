import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, ref, type Ref } from "vue";

type ApiError = {
  message: string;
  response?: { statusText?: string; data?: unknown };
};

describe("setupApiNotifications", () => {
  let addNotification: ReturnType<typeof vi.fn>;
  let lastApiError: Ref<ApiError | undefined>;
  let lastConnectionError: Ref<string | undefined>;
  let setupApiNotifications: () => void;

  beforeEach(async () => {
    vi.resetModules();

    addNotification = vi.fn();
    lastApiError = ref<ApiError | undefined>(undefined);
    lastConnectionError = ref<string | undefined>(undefined);

    vi.doMock("@agdb-studio/api/src/api", () => {
      return {
        lastApiError,
        lastConnectionError,
      };
    });

    vi.doMock(
      "@agdb-studio/notification/src/composables/notificationStore",
      () => {
        return {
          addNotification,
        };
      },
    );

    ({ setupApiNotifications } = await import("./apiNotifications"));
    setupApiNotifications();
  });

  it("shows response error notification", async () => {
    lastApiError.value = {
      message: "network error",
      response: { statusText: "Bad Request", data: "invalid payload" },
    };

    await nextTick();

    expect(addNotification).toHaveBeenCalledWith({
      type: "error",
      title: "Error: Bad Request",
      message: "invalid payload",
    });
  });

  it("shows generic error notification", async () => {
    lastApiError.value = {
      message: "connection failed",
    };

    await nextTick();

    expect(addNotification).toHaveBeenCalledWith({
      type: "error",
      title: "Error",
      message: "connection failed",
    });
  });

  it("shows final connection failure notification", async () => {
    lastConnectionError.value = "Connection attempt 20 failed.";

    await nextTick();

    expect(addNotification).toHaveBeenCalledWith({
      type: "error",
      title: "Connection error",
      message: "Connection attempt 20 failed.",
    });
  });

  it("does not notify when both signals are empty", async () => {
    await nextTick();
    expect(addNotification).not.toHaveBeenCalled();
  });

  it("does not notify when api error is cleared", async () => {
    lastApiError.value = {
      message: "temporary failure",
    };
    await nextTick();
    expect(addNotification).toHaveBeenCalledTimes(1);

    lastApiError.value = undefined;
    await nextTick();

    expect(addNotification).toHaveBeenCalledTimes(1);
  });

  it("does not notify when connection error is cleared", async () => {
    lastConnectionError.value = "Connection attempt 1 failed.";
    await nextTick();
    expect(addNotification).toHaveBeenCalledTimes(1);

    lastConnectionError.value = undefined;
    await nextTick();

    expect(addNotification).toHaveBeenCalledTimes(1);
  });
});
