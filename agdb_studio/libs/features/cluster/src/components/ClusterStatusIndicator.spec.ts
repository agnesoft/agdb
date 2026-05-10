import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick, ref } from "vue";
import ClusterStatusIndicator from "./ClusterStatusIndicator.vue";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

const mockServers = ref<ClusterStatus[]>([]);
const mockOverallStatus = ref<"red" | "amber" | "green" | "unknown">("unknown");
const mockIsLoading = ref(false);
const mockFetchStatus = vi.fn();
const mockSwitchingServerAddress = ref<string | null>(null);
const mockSwitchToServer = vi.fn();
const mockIsServerActive = vi.fn(() => false);
const mockIsUserLoggedInOnServer = vi.fn<
  (server: ClusterStatus) => boolean | null
>(() => null);
const mockActiveServer = ref<ClusterStatus | undefined>(undefined);
const mockActiveNodeLabel = ref(":3000");
vi.mock("../composables/clusterStatus", () => ({
  useClusterStatus: () => ({
    servers: mockServers,
    overallStatus: mockOverallStatus,
    isLoading: mockIsLoading,
    fetchStatus: mockFetchStatus,
    switchingServerAddress: mockSwitchingServerAddress,
    switchToServer: mockSwitchToServer,
    isServerActive: mockIsServerActive,
    isUserLoggedInOnServer: mockIsUserLoggedInOnServer,
    activeServer: mockActiveServer,
    activeNodeLabel: mockActiveNodeLabel,
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
    mockSwitchingServerAddress.value = null;
    mockIsServerActive.mockReturnValue(false);
    mockIsUserLoggedInOnServer.mockReturnValue(null);
    mockActiveServer.value = undefined;
    mockActiveNodeLabel.value = ":3000";
  });

  it("should render status indicator with correct color for green status", () => {
    mockOverallStatus.value = "green";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.exists()).toBe(true);
    expect(indicator.classes()).toContain("green");
  });

  it("should render status indicator with correct color for amber status", () => {
    mockOverallStatus.value = "amber";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.classes()).toContain("amber");
  });

  it("should render status indicator with correct color for red status", () => {
    mockOverallStatus.value = "red";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    const indicator = wrapper.find(".status-indicator");
    expect(indicator.classes()).toContain("red");
  });

  it("should show details on mouse enter and fetch status", async () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
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
          FadeTransition: { template: "<div><slot /></div>" },
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
          FadeTransition: { template: "<div><slot /></div>" },
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
          FadeTransition: { template: "<div><slot /></div>" },
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
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    expect(wrapper.find(".no-servers").exists()).toBe(true);
    expect(wrapper.find(".no-servers").text()).toBe("No clusters found");
  });

  it("should render server list with online servers", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: false },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers).toHaveLength(2);
    expect(servers[0]?.text()).toContain("server1:8080");
    expect(servers[1]?.text()).toContain("server2:8080");
  });

  it("should show crown icon for leader server", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    const crownIcons = wrapper.findAll("[data-testid='crown-icon']");

    expect(crownIcons).toHaveLength(1);
    expect(servers[0]?.find("[data-testid='crown-icon']").exists()).toBe(true);
    expect(servers[1]?.find("[data-testid='crown-icon']").exists()).toBe(false);
  });

  it("should mark offline servers with offline class", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: false, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers[0]?.classes()).not.toContain("offline");
    expect(servers[1]?.classes()).toContain("offline");
  });

  it("should call switchToServer when server row is clicked", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    await servers[1]!.trigger("click");

    expect(mockSwitchToServer).toHaveBeenCalledWith(mockServers.value[1]);
  });

  it("should apply logged in/out classes for server login icon", async () => {
    mockIsLoading.value = false;
    mockIsUserLoggedInOnServer.mockImplementation(
      (server: ClusterStatus) => server.address === "server1:8080",
    );
    mockServers.value = [
      { address: "server1:8080", status: true, leader: false },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers[0]?.find(".server-login")?.classes()).toContain("loggedIn");
    expect(servers[1]?.find(".server-login")?.classes()).toContain("loggedOut");
  });

  it("should show connecting status text on server row", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];
    mockSwitchingServerAddress.value = "server2:8080";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    expect(servers[1]?.attributes("title")).toContain(
      "Node status: Connecting...",
    );
  });

  it("should show active status text for active server", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
    ];
    mockIsServerActive.mockImplementation(() => true);

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const server = wrapper.find(".server-item");
    expect(server.attributes("title")).toContain("Node status: Active");
  });

  it("should render crown icon for connected leader server", () => {
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];
    mockActiveServer.value = {
      address: "server1:8080",
      status: true,
      leader: true,
    };

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="active-server-crown-icon"></div>',
          },
        },
      },
    });

    expect(
      wrapper.find("[data-testid='active-server-crown-icon']").exists(),
    ).toBe(true);
  });

  it("should call switchToServer on keyboard space for clickable server", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    await servers[1]!.trigger("keydown.space");

    expect(mockSwitchToServer).toHaveBeenCalledWith(mockServers.value[1]);
  });

  it("should not call switchToServer for non-clickable server", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];
    mockSwitchingServerAddress.value = "server2:8080";

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    await servers[1]!.trigger("click");

    expect(mockSwitchToServer).not.toHaveBeenCalled();
  });

  it("should call switchToServer on keyboard enter for clickable server", async () => {
    mockIsLoading.value = false;
    mockServers.value = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    const wrapper = mount(ClusterStatusIndicator, {
      global: {
        stubs: {
          CrownIcon: CrownIconStub,
          FadeTransition: { template: "<div><slot /></div>" },
          PhFillCrownSimple: {
            template: '<div data-testid="crown-icon"></div>',
          },
        },
      },
    });

    await wrapper.trigger("mouseenter");
    await nextTick();

    const servers = wrapper.findAll(".server-item");
    await servers[1]!.trigger("keydown.enter");

    expect(mockSwitchToServer).toHaveBeenCalledWith(mockServers.value[1]);
  });
});
