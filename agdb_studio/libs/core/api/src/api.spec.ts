import { MAX_CONNECTION_ATTEMPTS } from "./constants";
import {
  client as apiClient,
  apiUrl,
  lastApiError,
  lastConnectionError,
  initClient,
  onApiError,
  removeToken,
  checkClient,
  quietClient,
  reconnectClient,
  setOnUnauthorized,
  resetUnauthorized,
  StudioClient,
  type StudioApi,
} from "./api";
import { api, newApi } from "@agdb-studio/testing/mocks/apiMock";
import { AgdbApiError } from "@agnesoft/agdb_api";
import { vi, describe, it, beforeEach, afterEach, expect } from "vitest";
import type { ComputedRef } from "vue";

const onUnauthorizedMock = vi.fn();
setOnUnauthorized(onUnauthorizedMock);

const respond = (status: number, body = "") =>
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response(body || null, { status })),
  );

describe("client service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetUnauthorized();
    api.token = undefined;
    Object.defineProperty(window, "location", {
      value: { pathname: "/studio", reload: vi.fn() },
      configurable: true,
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe("client.value", () => {
    it("returns client", () => {
      expect(apiClient.value).toBeDefined();
    });
  });
  describe("initClient", () => {
    it("retries a server that does not respond", async () => {
      vi.useFakeTimers();
      api.status.mockRejectedValue(new AgdbApiError(500, "down"));
      await initClient();
      expect(newApi).toHaveBeenCalledTimes(1);
      await vi.runAllTimersAsync();
      expect(newApi).toHaveBeenCalledTimes(MAX_CONNECTION_ATTEMPTS + 1);
      expect(lastConnectionError.value).toContain("This is the final attempt.");
      expect(apiClient.value).toBeUndefined();

      api.status.mockResolvedValue(200);
      vi.useRealTimers();
    });
    it("reports the errors of a connected client", async () => {
      await initClient();
      expect(apiClient.value?.client.quiet).toBe(false);
      expect(lastConnectionError.value).toBeUndefined();
    });
  });
  describe("StudioClient", () => {
    it("passes the responses on", async () => {
      respond(200, '{"a":1}');
      const client = new StudioClient();
      expect(await client.get("http://x/a", "t")).toEqual([200, { a: 1 }]);
      expect(await client.post("http://x/a", {}, "t")).toEqual([200, { a: 1 }]);
      expect(await client.put("http://x/a", {}, "t")).toBe(200);
      expect(await client.delete("http://x/a", "t")).toBe(200);
    });
    it("reports an error and rethrows it", async () => {
      respond(404, "not found");
      const error = await new StudioClient()
        .get("http://x/a", "t")
        .catch((e: unknown) => e);
      expect(error).toEqual(new AgdbApiError(404, "not found"));
      expect(lastApiError.value).toEqual(new AgdbApiError(404, "not found"));
    });
    it("leaves the errors of a quiet client to its callers", async () => {
      respond(409, "quiet error");
      await expect(
        new StudioClient(true).put("http://x/a", {}, "t"),
      ).rejects.toEqual(new AgdbApiError(409, "quiet error"));
      expect(lastApiError.value?.description).not.toBe("quiet error");
    });
  });
  describe("onApiError", () => {
    it("removes the token on 401 without setting lastApiError", () => {
      onApiError(new AgdbApiError(401, "unauthorized"), false);
      expect(onUnauthorizedMock).toHaveBeenCalled();
      expect(lastApiError.value?.description).not.toBe("unauthorized");
    });
    it("reports other errors as errors of status 500", () => {
      onApiError(new SyntaxError("bad json"), false);
      expect(lastApiError.value).toEqual(
        new AgdbApiError(500, "SyntaxError: bad json"),
      );
    });
    it("suppresses lastApiError when unauthorized", () => {
      onApiError(new AgdbApiError(401, "unauthorized"), false);
      onApiError(new AgdbApiError(404, "not found after logout"), false);
      expect(lastApiError.value?.description).not.toBe(
        "not found after logout",
      );
    });
  });
  describe("quietClient", () => {
    it("copies the server and the token into a quiet client", () => {
      api.token = "token";
      quietClient(api as unknown as StudioApi);
      expect(newApi).toHaveBeenCalledTimes(1);
    });
  });
  describe("checkClient", () => {
    it("throws error if client is not initialized", () => {
      expect(() => {
        checkClient({ value: undefined } as unknown as ComputedRef<
          StudioApi | undefined
        >);
      }).toThrow("Client is not initialized");
    });
    it("does not throw error if client is initialized", () => {
      expect(() => {
        checkClient({ value: {} as StudioApi } as unknown as ComputedRef<
          StudioApi | undefined
        >);
      }).not.toThrow();
    });
  });
  describe("removeToken", () => {
    it("drops the token and calls onUnauthorized callback", async () => {
      await initClient();
      api.token = "token";
      removeToken();
      expect(api.token).toBeUndefined();
      expect(onUnauthorizedMock).toHaveBeenCalled();
    });
    it("handles undefined client", async () => {
      api.status.mockRejectedValueOnce(new AgdbApiError(500, "down"));
      vi.useFakeTimers();
      await initClient();
      expect(() => removeToken()).not.toThrow();
      expect(onUnauthorizedMock).toHaveBeenCalled();
      vi.clearAllTimers();
      vi.useRealTimers();
    });
  });
  describe("notifyUnauthorized deduplication", () => {
    it("fires callback only once until reset", () => {
      removeToken();
      expect(onUnauthorizedMock).toHaveBeenCalledTimes(1);

      removeToken();
      expect(onUnauthorizedMock).toHaveBeenCalledTimes(1);
    });

    it("fires callback again after resetUnauthorized", () => {
      removeToken();
      expect(onUnauthorizedMock).toHaveBeenCalledTimes(1);

      resetUnauthorized();

      removeToken();
      expect(onUnauthorizedMock).toHaveBeenCalledTimes(2);
    });
  });
  describe("reconnectClient", () => {
    it("reconnects to new url with the token of the client", async () => {
      await initClient();
      api.token = "test-token";
      await reconnectClient("http://localhost:3000");

      expect(newApi).toHaveBeenLastCalledWith(
        expect.any(StudioClient),
        "http://localhost:3000",
      );
      expect(api.token).toBe("test-token");
      expect(apiUrl.value).toBe("http://localhost:3000");
    });

    it("reconnects without token if not present", async () => {
      await initClient();
      localStorage.removeItem("studio_token");
      await reconnectClient("http://localhost:3000");

      expect(api.token).toBeUndefined();
    });

    it("reconnects with token from localStorage if client has no token", async () => {
      await initClient();
      localStorage.setItem("studio_token", "stored-token");
      await reconnectClient("http://localhost:3000");

      expect(api.token).toBe("stored-token");
      localStorage.removeItem("studio_token");
    });
  });
  describe("apiUrl", () => {
    it("returns api url", () => {
      expect(typeof apiUrl.value).toBe("string");
    });
  });
});
