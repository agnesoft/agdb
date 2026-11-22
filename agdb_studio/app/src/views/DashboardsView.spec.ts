import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick, reactive } from "vue";

const { route, mounted } = vi.hoisted(() => ({
  route: { value: undefined as unknown as { name: string } },
  mounted: vi.fn(),
}));
vi.mock("vue-router", () => ({ useRoute: () => route.value }));
vi.mock("@agdb-studio/dashboard/src/components/DashboardList.vue", () => ({
  default: {
    name: "DashboardList",
    mounted,
    template: '<div class="list" />',
  },
}));

import DashboardsView from "./DashboardsView.vue";

describe("DashboardsView", () => {
  it("renders the dashboard list", () => {
    route.value = reactive({ name: "dashboards" });
    expect(mount(DashboardsView).find(".list").exists()).toBe(true);
  });

  it("starts over when switching between the admin and user pages", async () => {
    route.value = reactive({ name: "admin-dashboards" });
    mounted.mockClear();
    mount(DashboardsView);
    expect(mounted).toHaveBeenCalledTimes(1);
    route.value.name = "dashboards";
    await nextTick();
    expect(mounted).toHaveBeenCalledTimes(2);
  });
});
