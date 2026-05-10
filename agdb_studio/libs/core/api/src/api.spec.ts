import { MAX_CONNECTION_ATTEMPTS } from "@/constants";
import {
  client as apiClient,
  apiUrl,
  initClient,
  responseInterceptor,
  errorInterceptor,
  removeToken,
  checkClient,
  reconnectClient,
} from "./api";
import { client } from "@agdb-studio/testing/mocks/apiMock";
import type { AxiosError, AxiosResponse } from "axios";
import { vi, describe, it, beforeEach, expect } from "vitest";
import type { ComputedRef } from "vue";
import type { AgdbApiClient } from "@agnesoft/agdb_api/client";

describe("client service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(window, "location", {
      value: { pathname: "/studio", reload: vi.fn() },
      configurable: true,
    });
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
    it("does not throw error if client is initialized", () => {
      expect(() => {
        checkClient({ value: {} as AgdbApiClient } as unknown as ComputedRef<
          AgdbApiClient | undefined
        >);
      }).not.toThrow();
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
    it("does not reload on login page", () => {
      Object.defineProperty(window, "location", {
        value: { pathname: "/studio/login", reload: vi.fn() },
        configurable: true,
      });
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
      expect(window.location.reload).not.toHaveBeenCalled();
    });
  });
  describe("reconnectClient", () => {
    it("reconnects to new url and attaches interceptors", async () => {
      const mockExistingClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        get_token: vi.fn().mockReturnValue("test-token"),
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      const mockNewClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      client
        .mockResolvedValueOnce(mockExistingClient)
        .mockResolvedValueOnce(mockNewClient);

      await initClient();
      await reconnectClient("http://localhost:3000");

      expect(client).toHaveBeenCalledWith("http://localhost:3000");
      expect(mockNewClient.set_token).toHaveBeenCalledWith("test-token");
      expect(mockNewClient.interceptors.response.use).toHaveBeenCalledWith(
        responseInterceptor,
        errorInterceptor,
      );
    });

    it("reconnects without token if not present", async () => {
      const mockExistingClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        get_token: vi.fn().mockReturnValue(null),
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      const mockNewClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      client
        .mockResolvedValueOnce(mockExistingClient)
        .mockResolvedValueOnce(mockNewClient);

      await initClient();
      localStorage.removeItem("studio_token");
      await reconnectClient("http://localhost:3000");

      expect(mockNewClient.set_token).not.toHaveBeenCalled();
    });

    it("reconnects with token from localStorage if client has no token", async () => {
      const mockExistingClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        get_token: vi.fn().mockReturnValue(null),
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      const mockNewClient = {
        interceptors: {
          response: {
            use: vi.fn(),
          },
          request: {
            use: vi.fn(),
          },
        },
        set_token: vi.fn(),
      } as unknown as AgdbApiClient;

      localStorage.setItem("studio_token", "stored-token");
      client
        .mockResolvedValueOnce(mockExistingClient)
        .mockResolvedValueOnce(mockNewClient);

      await initClient();
      await reconnectClient("http://localhost:3000");

      expect(mockNewClient.set_token).toHaveBeenCalledWith("stored-token");
      localStorage.removeItem("studio_token");
    });
  });
  describe("apiUrl", () => {
    it("returns api url", () => {
      expect(apiUrl.value).toBeDefined();
      expect(typeof apiUrl.value).toBe("string");
    });
  });
});
