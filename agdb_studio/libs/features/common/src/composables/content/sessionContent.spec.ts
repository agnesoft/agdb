import { describe, it, expect, vi } from "vitest";
import { buildSessionContent } from "./sessionContent";

describe("buildSessionContent", () => {
  it("returns no sessions message when empty", () => {
    const content = buildSessionContent([]);
    expect(content).toHaveLength(1);
    expect(content[0]?.paragraph?.[0]?.text).toBe("No active sessions.");
  });

  it("returns a table with session data", () => {
    const sessions = [
      { agent: "browser", created: 1000, expires_at: 2000 },
      { agent: "cli", created: 1100, expires_at: 2100 },
    ];
    const content = buildSessionContent(sessions);
    expect(content).toHaveLength(1);
    expect(content[0]?.table?.headers).toEqual(["Agent", "Created", "Expires"]);
    expect(content[0]?.table?.rows).toHaveLength(2);
    expect(content[0]?.table?.rows[0]?.[0]).toBe("browser");
  });

  it("uses (unknown) for empty agent", () => {
    const sessions = [{ agent: "", created: 1000, expires_at: 2000 }];
    const content = buildSessionContent(sessions);
    expect(content[0]?.table?.rows[0]?.[0]).toBe("(unknown)");
  });

  it("does not include rowActions when no onLogout provided", () => {
    const sessions = [{ agent: "browser", created: 1000, expires_at: 2000 }];
    const content = buildSessionContent(sessions);
    expect(content[0]?.table?.rowActions).toBeUndefined();
  });

  it("includes rowActions when onLogout provided", () => {
    const onLogout = vi.fn();
    const sessions = [{ agent: "browser", created: 1000, expires_at: 2000 }];
    const content = buildSessionContent(sessions, onLogout);
    expect(content[0]?.table?.rowActions).toHaveLength(1);
    expect(content[0]?.table?.rowActions?.[0]?.label).toBe("Logout");
    content[0]?.table?.rowActions?.[0]?.onClick(0);
    expect(onLogout).toHaveBeenCalledWith(0);
  });
});
