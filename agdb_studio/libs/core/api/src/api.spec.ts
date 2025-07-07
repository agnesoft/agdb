import { MAX_CONNECTION_ATTEMPTS } from "@/constants";
import {
  client as apiClient,
  initClient,
  responseInterceptor,
  errorInterceptor,
  removeToken,
  checkClient,
} from "./api";
import { client } from "@agdb-studio/testing/mocks/apiMock";
import type { AxiosError, AxiosResponse } from "axios";
import { vi, describe, it, beforeEach, expect } from "vitest";
import type { ComputedRef } from "vue";
import type { AgdbApiClient } from "@agnesoft/agdb_api/client";

describe("client service", () => {
  Object.defineProperty(window, "location", {
    value: { reload: vi.fn() },
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("client.value", () => {
    it("returns client", () => {
      expect(apiClient.value).toBeDefined();
    });
  });
  describe("initClient", () => {
    it("catches axios errors", async () => {
      vi.useFakeTimers();
      client.mockRejectedValue("error");
      await initClient().catch((error) => {
        expect(error).toBe("error");
      });
      expect(client).toHaveBeenCalledTimes(1);
      await vi.runAllTimersAsync();
      expect(client).toHaveBeenCalledTimes(MAX_CONNECTION_ATTEMPTS + 1);

      vi.useRealTimers();
    });
  });
  describe("responseInterceptor", () => {
    it("returns response", () => {
      const response = {
        data: "response",
        status: 200,
        statusText: "OK",
      };
      expect(
        responseInterceptor(response as unknown as AxiosResponse<string>),
      ).toBe(response);
    });
  });
  describe("errorInterceptor", () => {
    it("returns error for 401 response", async () => {
      const response = {
        message: "error",
        response: { status: 401 },
      };
      await expect(
        errorInterceptor(response as unknown as AxiosError<string>),
      ).rejects.toBe(response);
    });
    it("returns error for unknown response", async () => {
      const response = {
        message: "error",
      };
      await expect(
        errorInterceptor(response as unknown as AxiosError<string>),
      ).rejects.toBe(response);
    });
  });
  describe("checkClient", () => {
    it("throws error if client is not initialized", () => {
      expect(() => {
        checkClient({ value: undefined } as unknown as ComputedRef<
          AgdbApiClient | undefined
        >);
      }).toThrow("Client is not initialized");
    });
  });
  describe("removeToken", () => {
    it("reloads the page", () => {
      client.mockResolvedValue({
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        reset_token: vi.fn(),
      } as unknown as AgdbApiClient);
      initClient();
      removeToken();
      expect(window.location.reload).toHaveBeenCalled();
    });
    it("handles undefined client", () => {
      client.mockResolvedValue(undefined);
      initClient();
      expect(() => removeToken()).not.toThrow();
      expect(window.location.reload).toHaveBeenCalled();
    });
  });
});
