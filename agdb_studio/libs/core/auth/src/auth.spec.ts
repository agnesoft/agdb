import { useAuth, setLocalStorageToken, refreshToken } from "./auth";
import * as api from "@agdb-studio/api/src/api";
import { resetUnauthorized } from "@agdb-studio/api/src/api";
import {
  api as mockApi,
  cluster_user_login,
  cluster_user_logout,
  cluster_user_logout_all,
  cluster_user_logout_others,
  cluster_user_logout_session,
  user_login,
  user_logout,
  user_logout_all,
  user_logout_others,
  user_logout_session,
} from "@agdb-studio/testing/mocks/apiMock";
import { AgdbApiError } from "@agnesoft/agdb_api";
import { ACCESS_TOKEN } from "@agdb-studio/api/src/constants";
import { vi, describe, it, beforeEach, expect } from "vitest";

const onUnauthorizedMock = vi.fn();
api.setOnUnauthorized(onUnauthorizedMock);

describe("auth service", () => {
  Object.defineProperty(window, "location", {
    value: { reload: vi.fn() },
  });

  const { isLoggedIn, logout, logoutSession, login, token } = useAuth();
  beforeEach(() => {
    localStorage.removeItem(ACCESS_TOKEN);
    resetUnauthorized();
  });
  describe("isLoggedIn", () => {
    beforeEach(() => {
      localStorage.removeItem(ACCESS_TOKEN);
    });
    it("returns false if no token", () => {
      expect(isLoggedIn.value).toBe(false);
    });
    it("returns true if token", () => {
      setLocalStorageToken("test");
      expect(isLoggedIn.value).toBe(true);
      expect(token.value).toBe("test");
    });
  });
  describe("logout", () => {
    Object.defineProperty(window, "location", {
      value: { reload: vi.fn() },
    });
    beforeEach(() => {
      localStorage.removeItem(ACCESS_TOKEN);
      refreshToken();
    });
    it("does nothing if not logged in", async () => {
      await logout();
      expect(isLoggedIn.value).toBe(false);
    });
    it("logs out if logged in", async () => {
      setLocalStorageToken("test");
      await logout();
      expect(isLoggedIn.value).toBe(false);
    });
    it("should handle when server logout fails", async () => {
      user_logout.mockRejectedValueOnce(new Error("Logout failed"));
      setLocalStorageToken("test");
      await logout();
      expect(isLoggedIn.value).toBe(false);
    });
    it("logs out of the cluster", async () => {
      setLocalStorageToken("test");
      await logout(true);
      expect(cluster_user_logout).toHaveBeenCalled();
      expect(isLoggedIn.value).toBe(false);
    });
    it("skips the server logout", async () => {
      user_logout.mockClear();
      setLocalStorageToken("test");
      await logout(false, false);
      expect(user_logout).not.toHaveBeenCalled();
      expect(isLoggedIn.value).toBe(false);
    });
  });
  describe("login", () => {
    beforeEach(() => {
      mockApi.token = undefined;
    });
    it("returns token on success", async () => {
      user_login.mockImplementationOnce(async () => {
        mockApi.token = "token";
        return 200;
      });
      const token = await login({ username: "test", password: "test" });
      expect(user_login).toHaveBeenCalledWith({
        username: "test",
        password: "test",
      });
      expect(token).toBe("token");
      expect(localStorage.getItem(ACCESS_TOKEN)).toBe("token");
    });
    it("logs in to the cluster", async () => {
      cluster_user_login.mockImplementationOnce(async () => {
        mockApi.token = "cluster-token";
        return 200;
      });
      const token = await login({
        username: "test",
        password: "test",
        cluster: true,
      });
      expect(cluster_user_login).toHaveBeenCalledWith({
        username: "test",
        password: "test",
      });
      expect(token).toBe("cluster-token");
    });
    it("returns no token when the server gives none", async () => {
      const token = await login({ username: "test", password: "test" });
      expect(token).toBeUndefined();
    });
    it("switches api client when server is provided", async () => {
      const reconnectSpy = vi.spyOn(api, "reconnectClient");
      await login({
        username: "test",
        password: "test",
        server: "https://localhost:3001",
      });
      expect(reconnectSpy).toHaveBeenCalled();
    });
    it("throws error on failure", async () => {
      const error = new AgdbApiError(401, "unauthorized");
      user_login.mockRejectedValueOnce(error);
      await expect(login({ username: "test", password: "test" })).rejects.toBe(
        error,
      );
    });
    it("returns undefined without a client", async () => {
      vi.useFakeTimers();
      mockApi.status.mockRejectedValueOnce(new AgdbApiError(500, "down"));
      await api.initClient();
      expect(await login({ username: "test", password: "test" })).toBe(
        undefined,
      );
      setLocalStorageToken("test");
      await logoutSession();
      expect(user_logout).not.toHaveBeenCalled();
      await api.initClient();
      vi.clearAllTimers();
      vi.useRealTimers();
    });
  });
  describe("setLocalStorageToken", () => {
    it("sets token", () => {
      setLocalStorageToken("test");
      expect(localStorage.getItem(ACCESS_TOKEN)).toBe("test");
    });
  });
  describe("refreshToken", () => {
    beforeEach(() => {
      localStorage.removeItem(ACCESS_TOKEN);
    });
    it("refreshes token", () => {
      setLocalStorageToken("test");
      expect(isLoggedIn.value).toBe(true);
    });
    it("calls notifyUnauthorized when transitioning to logged out", () => {
      refreshToken();
      expect(isLoggedIn.value).toBe(false);
      expect(onUnauthorizedMock).toHaveBeenCalled();
    });
  });
  describe("logoutSession", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      localStorage.removeItem(ACCESS_TOKEN);
      refreshToken();
    });
    it("does nothing if not logged in", async () => {
      await logoutSession();
      expect(user_logout).not.toHaveBeenCalled();
    });
    it("clears token when no session specified (current session)", async () => {
      setLocalStorageToken("test");
      await logoutSession();
      expect(isLoggedIn.value).toBe(false);
      expect(user_logout).toHaveBeenCalledWith();
    });
    it("clears token when session is 'all'", async () => {
      setLocalStorageToken("test");
      await logoutSession(false, "all");
      expect(isLoggedIn.value).toBe(false);
      expect(user_logout_all).toHaveBeenCalled();
    });
    it("keeps token when session is 'others'", async () => {
      setLocalStorageToken("test");
      await logoutSession(false, "others");
      expect(isLoggedIn.value).toBe(true);
      expect(user_logout_others).toHaveBeenCalled();
    });
    it("keeps token when logging out a specific session", async () => {
      setLocalStorageToken("test");
      await logoutSession(false, "session-123");
      expect(isLoggedIn.value).toBe(true);
      expect(user_logout_session).toHaveBeenCalledWith({
        session: "session-123",
      });
    });
    it("uses cluster endpoints when cluster is true", async () => {
      setLocalStorageToken("test");
      await logoutSession(true, "others");
      await logoutSession(true, "session-123");
      await logoutSession(true, "all");
      expect(cluster_user_logout_others).toHaveBeenCalled();
      expect(cluster_user_logout_session).toHaveBeenCalledWith({
        session: "session-123",
      });
      expect(cluster_user_logout_all).toHaveBeenCalled();
      expect(user_logout_others).not.toHaveBeenCalled();
    });
    it("handles logout failure gracefully", async () => {
      user_logout_others.mockRejectedValueOnce(new Error("Network error"));
      setLocalStorageToken("test");
      await logoutSession(false, "others");
      expect(isLoggedIn.value).toBe(true);
    });
  });
});
