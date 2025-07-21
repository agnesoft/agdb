import { createLogger } from "./logger";
import { describe, it, expect, vi } from "vitest";

describe("Logger", () => {
  it("should log messages with the correct format", () => {
    const logger = createLogger("Test");

    const consoleLogSpy = vi.spyOn(console, "log");
    const consoleWarnSpy = vi.spyOn(console, "warn");
    const consoleErrorSpy = vi.spyOn(console, "error");
    const consoleDebugSpy = vi.spyOn(console, "debug");

    logger.info("This is an info message");
    logger.warn("This is a warning message");
    logger.error("This is an error message");
    logger.debug("This is a debug message");

    expect(consoleLogSpy).toHaveBeenCalledWith(
      "[Test] [INFO] This is an info message",
    );
    expect(consoleWarnSpy).toHaveBeenCalledWith(
      "[Test] [WARN] This is a warning message",
    );
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "[Test] [ERROR] This is an error message",
    );
    expect(consoleDebugSpy).toHaveBeenCalledWith(
      "[Test] [DEBUG] This is a debug message",
    );

    consoleLogSpy.mockRestore();
    consoleWarnSpy.mockRestore();
    consoleErrorSpy.mockRestore();
    consoleDebugSpy.mockRestore();
  });

  it("should log messages without prefix when no prefix is provided", () => {
    const logger = createLogger();

    const consoleLogSpy = vi.spyOn(console, "log");
    const consoleWarnSpy = vi.spyOn(console, "warn");
    const consoleErrorSpy = vi.spyOn(console, "error");
    const consoleDebugSpy = vi.spyOn(console, "debug");

    logger.info("This is an info message");
    logger.warn("This is a warning message");
    logger.error("This is an error message");
    logger.debug("This is a debug message");

    expect(consoleLogSpy).toHaveBeenCalledWith(
      "[INFO] This is an info message",
    );
    expect(consoleWarnSpy).toHaveBeenCalledWith(
      "[WARN] This is a warning message",
    );
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "[ERROR] This is an error message",
    );
    expect(consoleDebugSpy).toHaveBeenCalledWith(
      "[DEBUG] This is a debug message",
    );

    consoleLogSpy.mockRestore();
    consoleWarnSpy.mockRestore();
    consoleErrorSpy.mockRestore();
    consoleDebugSpy.mockRestore();
  });

  it("should handle multiple message types", () => {
    const logger = createLogger("Test");

    const consoleLogSpy = vi.spyOn(console, "log");

    logger.info("Info message", 123, true, null, undefined);

    expect(consoleLogSpy).toHaveBeenCalledWith(
      "[Test] [INFO] Info message 123 true  ",
    );

    consoleLogSpy.mockRestore();
  });
});
