import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick, reactive } from "vue";

const { useRoute, mounted } = vi.hoisted(() => ({
  useRoute: vi.fn(),
  mounted: vi.fn(),
}));
vi.mock("vue-router", () => ({ useRoute }));
vi.mock("@agdb-studio/dashboard/src/components/DashboardView.vue", () => ({
  default: {
    name: "DashboardView",
    props: ["name"],
    mounted,
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

  it("starts over when switching between the admin and user pages", async () => {
    const route = reactive({
      name: "admin-dashboard",
      params: { name: "ops" },
    });
    useRoute.mockReturnValue(route);
    mounted.mockClear();
    mount(DashboardView);
    expect(mounted).toHaveBeenCalledTimes(1);
    route.name = "dashboard";
    await nextTick();
    expect(mounted).toHaveBeenCalledTimes(2);
  });
});
