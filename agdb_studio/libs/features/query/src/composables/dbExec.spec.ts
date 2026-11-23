import { beforeEach, describe, expect, it, vi } from "vitest";

const { routeState, routeControl, checkClient, client, quiet, quietClient } =
  vi.hoisted(() => {
    const exec = () => ({
      db_exec: vi.fn(),
      db_exec_mut: vi.fn(),
      admin_db_exec: vi.fn(),
      admin_db_exec_mut: vi.fn(),
    });
    const quiet = exec();
    return {
      quiet,
      quietClient: vi.fn(() => quiet),
      client: { value: exec() },
      routeState: {
        params: { owner: "alice", db: "main" },
        name: "db-view",
      } as { params: Record<string, unknown>; name?: unknown },
      routeControl: { throwUseRoute: false, noRoute: false },
      checkClient: vi.fn(),
    };
  });

vi.mock("vue-router", () => ({
  useRoute: () => {
    if (routeControl.throwUseRoute) throw new Error("no route");
    return routeControl.noRoute ? undefined : routeState;
  },
}));

vi.mock("@agdb-studio/api/src/api", () => ({
  checkClient,
  client,
  quietClient,
}));

import { h, ref } from "vue";
import { mount } from "@vue/test-utils";
import {
  executeDbQueries,
  provideDbTarget,
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
    for (const fn of [
      ...Object.values(client.value),
      ...Object.values(quiet),
    ]) {
      fn.mockResolvedValue([200, data]);
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
    expect(client.value[fn]).toHaveBeenCalledWith({
      owner: "alice",
      db: "main",
      queries: [query],
    });
    expect(result).toBe(data);
    // A quiet run goes through a client without global error notifications.
    await executeDbQueries([query], mut, routeState, { quiet: true });
    expect(quietClient).toHaveBeenCalledWith(client.value);
    expect(quiet[fn]).toHaveBeenCalledWith({
      owner: "alice",
      db: "main",
      queries: [query],
    });
    expect(client.value[fn]).toHaveBeenCalledTimes(1);
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

  it("uses a provided database target instead of the route", async () => {
    routeState.name = "admin-db-view";
    const target = ref({ owner: "bob", db: "sales" });
    let bound: ReturnType<typeof useDbExec> | undefined;
    const Child = {
      setup() {
        bound = useDbExec();
        return () => null;
      },
    };
    mount({
      setup() {
        provideDbTarget(target);
        return () => h(Child);
      },
    });
    expect(bound!.route.name).toBe("query");
    await bound!.exec([query]);
    expect(client.value.db_exec).toHaveBeenCalledWith({
      owner: "bob",
      db: "sales",
      queries: [query],
    });
    target.value = { owner: "carol", db: "hr" };
    await bound!.execMut([query]);
    expect(client.value.db_exec_mut).toHaveBeenCalledWith({
      owner: "carol",
      db: "hr",
      queries: [query],
    });
    expect(client.value.admin_db_exec).not.toHaveBeenCalled();
  });

  it("runs an admin database target through the admin endpoints", async () => {
    const target = ref({ owner: "bob", db: "sales", admin: true });
    let bound: ReturnType<typeof useDbExec> | undefined;
    const Child = {
      setup() {
        bound = useDbExec();
        return () => null;
      },
    };
    mount({
      setup() {
        provideDbTarget(target);
        return () => h(Child);
      },
    });
    expect(bound!.route.name).toBe("admin-query");
    await bound!.exec([query]);
    expect(client.value.admin_db_exec).toHaveBeenCalledWith({
      owner: "bob",
      db: "sales",
      queries: [query],
    });
    target.value = { owner: "bob", db: "sales", admin: false };
    expect(bound!.route.name).toBe("query");
  });
});
