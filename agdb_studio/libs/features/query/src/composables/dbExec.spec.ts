import { beforeEach, describe, expect, it, vi } from "vitest";

const { routeState, routeControl, checkClient, client } = vi.hoisted(() => ({
  routeState: {
    params: { owner: "alice", db: "main" },
    name: "db-view",
  } as { params: Record<string, unknown>; name?: unknown },
  routeControl: { throwUseRoute: false, noRoute: false },
  checkClient: vi.fn(),
  client: {
    value: {
      db_exec: vi.fn(),
      db_exec_mut: vi.fn(),
      admin_db_exec: vi.fn(),
      admin_db_exec_mut: vi.fn(),
    },
  },
}));

vi.mock("vue-router", () => ({
  useRoute: () => {
    if (routeControl.throwUseRoute) throw new Error("no route");
    return routeControl.noRoute ? undefined : routeState;
  },
}));

vi.mock("@agdb-studio/api/src/api", () => ({ checkClient, client }));

import {
  executeDbQueries,
  getDbParams,
  isAdminRoute,
  useDbExec,
  useMinimalRoute,
} from "./dbExec";

const data = [{ result: 1, elements: [] }];
const query = { SelectNodeCount: {} } as never;

describe("dbExec", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    routeState.params = { owner: "alice", db: "main" };
    routeState.name = "db-view";
    routeControl.throwUseRoute = false;
    routeControl.noRoute = false;
    for (const fn of Object.values(client.value)) {
      fn.mockResolvedValue({ data });
    }
  });

  it("resolves db params from the route", () => {
    expect(getDbParams(routeState)).toEqual({ owner: "alice", db: "main" });
    expect(getDbParams({ params: { owner: 1, db: "x" } })).toBeUndefined();
    expect(getDbParams({ params: {} })).toBeUndefined();
  });

  it("detects admin routes", () => {
    expect(isAdminRoute({ params: {}, name: "admin-db" })).toBe(true);
    expect(isAdminRoute({ params: {}, name: "db" })).toBe(false);
    expect(isAdminRoute({ params: {} })).toBe(false);
  });

  it("falls back to an empty route outside of a router", () => {
    routeControl.throwUseRoute = true;
    expect(useMinimalRoute()).toEqual({ params: {}, name: undefined });
  });

  it("falls back to an empty route when no route is provided", () => {
    routeControl.noRoute = true;
    expect(useMinimalRoute()).toEqual({ params: {}, name: undefined });
  });

  it.each([
    ["db-view", false, "db_exec"],
    ["db-view", true, "db_exec_mut"],
    ["admin-db-view", false, "admin_db_exec"],
    ["admin-db-view", true, "admin_db_exec_mut"],
  ] as const)("on route %s (mutation %s) calls %s", async (name, mut, fn) => {
    routeState.name = name;
    const result = await executeDbQueries([query], mut, routeState);
    expect(checkClient).toHaveBeenCalled();
    expect(client.value[fn]).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [query],
    );
    expect(result).toBe(data);
  });

  it("returns undefined without db params", async () => {
    expect(await executeDbQueries([query], false, { params: {} })).toBe(
      undefined,
    );
    expect(client.value.db_exec).not.toHaveBeenCalled();
  });

  it("binds exec helpers to the current route", async () => {
    const { exec, execMut } = useDbExec();
    await exec([query]);
    await execMut([query]);
    expect(client.value.db_exec).toHaveBeenCalledTimes(1);
    expect(client.value.db_exec_mut).toHaveBeenCalledTimes(1);
  });
});
