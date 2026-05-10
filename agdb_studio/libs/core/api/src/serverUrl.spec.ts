import { describe, expect, it } from "vitest";
import { resolveServerUrl } from "./serverUrl";

describe("resolveServerUrl", () => {
  it("keeps localhost host and swaps to selected server port", () => {
    const resolved = resolveServerUrl(
      "http://localhost:3000",
      "https://agdb1:9090",
    );

    expect(resolved).toBe("http://localhost:9090");
  });

  it("works when current and target addresses omit protocol", () => {
    const resolved = resolveServerUrl("localhost:3000", "agdb1:9090");

    expect(resolved).toBe("http://localhost:9090");
  });

  it("keeps localhost host when selected server has no port", () => {
    const resolved = resolveServerUrl("http://localhost:3000", "agdb1");

    expect(resolved).toBe("http://localhost");
  });

  it("returns the target server url for non-localhost current host", () => {
    const resolved = resolveServerUrl(
      "http://server1:3000",
      "https://agdb1:9090",
    );

    expect(resolved).toBe("https://agdb1:9090");
  });

  it("inherits current protocol for target without protocol", () => {
    const resolved = resolveServerUrl("https://server1:3000", "agdb1:9090");

    expect(resolved).toBe("https://agdb1:9090");
  });

  it("returns serverAddress on parse failure", () => {
    const resolved = resolveServerUrl("http://%zz", "agdb1:9090");

    expect(resolved).toBe("agdb1:9090");
  });
});
