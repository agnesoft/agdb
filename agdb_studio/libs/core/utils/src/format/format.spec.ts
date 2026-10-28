import { describe, it, expect } from "vitest";
import { formatBytes, formatTimestamp, formatUptime } from "./format";

describe("formatBytes", () => {
  it("formats 0 bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
  });

  it("formats bytes", () => {
    expect(formatBytes(512)).toBe("512 B");
  });

  it("formats kilobytes", () => {
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatBytes(1048576)).toBe("1.0 MB");
    expect(formatBytes(268435456)).toBe("256.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatBytes(1073741824)).toBe("1.0 GB");
  });

  it("formats terabytes", () => {
    expect(formatBytes(1099511627776)).toBe("1.0 TB");
  });

  it("clamps to TB for petabyte-scale values", () => {
    expect(formatBytes(1125899906842624)).toBe("1024.0 TB");
  });

  it("returns 0 B for negative input", () => {
    expect(formatBytes(-100)).toBe("0 B");
  });
});

describe("formatTimestamp", () => {
  it("converts epoch seconds to locale string", () => {
    const result = formatTimestamp(0);
    expect(result).toBe(new Date(0).toLocaleString());
  });

  it("handles a specific timestamp", () => {
    const result = formatTimestamp(1700000000);
    expect(result).toBe(new Date(1700000000000).toLocaleString());
  });
});

describe("formatUptime", () => {
  it("formats zero seconds", () => {
    expect(formatUptime(0)).toBe("0s");
  });

  it("formats seconds only", () => {
    expect(formatUptime(45)).toBe("45s");
  });

  it("formats minutes and seconds", () => {
    expect(formatUptime(125)).toBe("2m 5s");
  });

  it("formats hours", () => {
    expect(formatUptime(3661)).toBe("1h 1m 1s");
  });

  it("formats days", () => {
    expect(formatUptime(90061)).toBe("1d 1h 1m 1s");
  });

  it("omits zero middle components", () => {
    expect(formatUptime(86400)).toBe("1d");
    expect(formatUptime(3600)).toBe("1h");
    expect(formatUptime(60)).toBe("1m");
  });

  it("formats large uptime", () => {
    expect(formatUptime(259200)).toBe("3d");
  });

  it("floors fractional seconds", () => {
    expect(formatUptime(90061.7)).toBe("1d 1h 1m 1s");
  });
});
