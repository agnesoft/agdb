import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";

vi.mock("@agdb-studio/dashboard/src/components/DashboardList.vue", () => ({
  default: { name: "DashboardList", template: '<div class="list" />' },
}));

import DashboardsView from "./DashboardsView.vue";

describe("DashboardsView", () => {
  it("renders the dashboard list", () => {
    expect(mount(DashboardsView).find(".list").exists()).toBe(true);
  });
});
