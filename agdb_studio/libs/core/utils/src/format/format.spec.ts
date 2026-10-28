import { describe, it, expect } from "vitest";
import {
  formatAgent,
  formatBytes,
  formatTimestamp,
  formatUptime,
} from "./format";

describe("formatAgent", () => {
  it("returns (unknown) for empty string", () => {
    expect(formatAgent("")).toBe("(unknown)");
  });

  it("passes through short agent strings", () => {
    expect(formatAgent("Firefox")).toBe("Firefox");
    expect(formatAgent("Chrome")).toBe("Chrome");
    expect(formatAgent("CLI")).toBe("CLI");
  });

  it("extracts Chrome from full UA", () => {
    expect(
      formatAgent(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
      ),
    ).toBe("Chrome");
  });

  it("extracts Firefox from full UA", () => {
    expect(
      formatAgent(
        "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
      ),
    ).toBe("Firefox");
  });

  it("extracts Safari from full UA", () => {
    expect(
      formatAgent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
      ),
    ).toBe("Safari");
  });

  it("extracts Edge from full UA", () => {
    expect(
      formatAgent(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
      ),
    ).toBe("Edge");
  });

  it("extracts Opera from full UA", () => {
    expect(
      formatAgent(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 OPR/106.0.0.0",
      ),
    ).toBe("Opera");
  });

  it("falls back to first word for unknown long UA", () => {
    expect(formatAgent("CustomBot/1.0 (compatible; crawler)")).toBe(
      "CustomBot",
    );
  });
});

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
