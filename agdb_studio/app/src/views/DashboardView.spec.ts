import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";

const { useRoute } = vi.hoisted(() => ({ useRoute: vi.fn() }));
vi.mock("vue-router", () => ({ useRoute }));
vi.mock("@agdb-studio/dashboard/src/components/DashboardView.vue", () => ({
  default: {
    name: "DashboardView",
    props: ["name"],
    template: '<div class="dashboard">{{ name }}</div>',
  },
}));

import DashboardView from "./DashboardView.vue";

describe("DashboardView", () => {
  it("passes the route name to the dashboard", () => {
    useRoute.mockReturnValue({ params: { name: "ops" } });
    expect(mount(DashboardView).find(".dashboard").text()).toBe("ops");
  });

  it("handles a non-string name", () => {
    useRoute.mockReturnValue({ params: { name: ["a"] } });
    expect(mount(DashboardView).find(".dashboard").text()).toBe("");
  });
});
