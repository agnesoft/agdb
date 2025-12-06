import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import ClusterStatusIndicator from "./ClusterStatusIndicator.vue";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

const mockServers = vi.hoisted(() => ({
  value: [] as ClusterStatus[],
}));

const mockOverallStatus = vi.hoisted(() => ({
  value: "unknown" as "red" | "amber" | "green" | "unknown",
}));

const mockIsLoading = vi.hoisted(() => ({
  value: false,
}));

const mockFetchStatus = vi.hoisted(() => vi.fn());

vi.mock("../composables/clusterStatus", () => ({
  useClusterStatus: () => ({
    servers: mockServers,
    overallStatus: mockOverallStatus,
    isLoading: mockIsLoading,
    fetchStatus: mockFetchStatus,
  }),
}));

const CrownIconStub = {
  name: "CrownIcon",
  template: '<svg class="crown-icon"></svg>',
};

describe("ClusterStatusIndicator", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockServers.value = [];
    mockOverallStatus.value = "unknown";
    mockIsLoading.value = false;
  });

  it("should render status indicator with correct color for green status", () => {
    mockOverallStatus.value = "green";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.exists()).toBe(true);
    expect(indicator.attributes("style")).toContain("var(--green-1)");
  });

  it("should render status indicator with correct color for amber status", () => {
    mockOverallStatus.value = "amber";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.attributes("style")).toContain("var(--orange-1)");
  });

  it("should render status indicator with correct color for red status", () => {
    mockOverallStatus.value = "red";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.attributes("style")).toContain("var(--red-1)");
  });

  it("should show details on mouse enter and fetch status", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    expect(wrapper.find(".status-details").exists()).toBe(false);

    await wrapper.trigger("mouseenter");
    await nextTick();

    expect(wrapper.find(".status-details").exists()).toBe(true);
    expect(mockFetchStatus).toHaveBeenCalled();
  });

  it("should hide details on mouse leave", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();
    expect(wrapper.find(".status-details").exists()).toBe(true);

    await wrapper.trigger("mouseleave");
    await nextTick();
    expect(wrapper.find(".status-details").exists()).toBe(false);
  });

  it("should toggle details on click", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    expect(wrapper.find(".status-details").exists()).toBe(false);

    await wrapper.trigger("click");
    await nextTick();
    expect(wrapper.find(".status-details").exists()).toBe(true);

    await wrapper.trigger("click");
    await nextTick();
    expect(wrapper.find(".status-details").exists()).toBe(false);
  });

  it("should display loading state", async () => {
    mockIsLoading.value = true;

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    expect(wrapper.find(".loading").exists()).toBe(true);
    expect(wrapper.find(".loading").text()).toBe("Loading...");
  });

  it("should display no servers message when empty", async () => {
    mockIsLoading.value = false;
    mockServers.value = [];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    expect(wrapper.find(".no-servers").exists()).toBe(true);
    expect(wrapper.find(".no-servers").text()).toBe("No servers found");
  });

  it("should render server list with online servers", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: false },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers).toHaveLength(2);
    expect(servers[0].text()).toContain("server1:8080");
    expect(servers[0].text()).toContain("Online");
    expect(servers[1].text()).toContain("server2:8080");
  });

  it("should show crown icon for leader server", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    const crownIcons = wrapper.findAll(".crown-icon");

    expect(crownIcons).toHaveLength(1);
    expect(servers[0].find(".crown-icon").exists()).toBe(true);
    expect(servers[1].find(".crown-icon").exists()).toBe(false);
  });

  it("should mark offline servers with offline class", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: false, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers[0].classes()).not.toContain("offline");
    expect(servers[1].classes()).toContain("offline");
    expect(servers[1].text()).toContain("Offline");
  });
});
