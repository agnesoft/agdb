import { afterEach, describe, expect, it, vi } from "vitest";
import { copyText } from "./clipboard";

describe("copyText", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("uses the Clipboard API when there is one", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    expect(await copyText("hi")).toBe(true);
    expect(writeText).toHaveBeenCalledWith("hi");

    writeText.mockRejectedValueOnce(new Error("denied"));
    expect(await copyText("hi")).toBe(false);
  });

  it("falls back to the copy command without the Clipboard API", async () => {
    vi.stubGlobal("navigator", {});
    let copied: string | undefined;
    const execCommand = vi.fn(() => {
      copied = document.querySelector("textarea")?.value;
      return true;
    });
    document.execCommand = execCommand;
    expect(await copyText("hi")).toBe(true);
    expect(execCommand).toHaveBeenCalledWith("copy");
    expect(copied).toBe("hi");
    // The helper element is removed again.
    expect(document.querySelector("textarea")).toBeNull();

    document.execCommand = vi.fn(() => {
      throw new Error("unsupported");
    });
    expect(await copyText("hi")).toBe(false);
  });
});
