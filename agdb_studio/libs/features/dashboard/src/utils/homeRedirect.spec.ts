import { beforeEach, describe, expect, it, vi } from "vitest";

const { fetchHome } = vi.hoisted(() => ({ fetchHome: vi.fn() }));
vi.mock("../composables/dashboardStore", () => ({
  useDashboardStore: () => ({ fetchHome }),
}));

import { homeRedirect } from "./homeRedirect";

describe("homeRedirect", () => {
  beforeEach(() => vi.clearAllMocks());

  it("opens the home dashboard", async () => {
    fetchHome.mockResolvedValueOnce({ owner: "ann", name: "ops" });
    expect(await homeRedirect()).toEqual({
      name: "dashboard",
      params: { owner: "ann", name: "ops" },
    });
  });

  it("opens the databases without a home dashboard", async () => {
    fetchHome.mockResolvedValueOnce(undefined);
    expect(await homeRedirect()).toEqual({ name: "db" });
  });

  it("opens the databases when the home cannot be fetched", async () => {
    fetchHome.mockRejectedValueOnce(new Error("offline"));
    expect(await homeRedirect()).toEqual({ name: "db" });
  });
});
