import { describe, expect, it } from "vitest";
import { createLinkGradientMaterials, splitAlpha } from "./linkGradient";

describe("linkGradient", () => {
  it.each([
    ["#abc", "#aabbcc", 1],
    ["#abcd", "#aabbcc", 0xdd / 255],
    ["#112233", "#112233", 1],
    ["#11223380", "#112233", 0x80 / 255],
    ["rgb(1, 2, 3)", "rgb(1, 2, 3)", 1],
    ["rgba(1, 2, 3, 0.25)", "rgb(1, 2, 3)", 0.25],
    ["rgb(1 2 3 / 50%)", "rgb(1, 2, 3)", 0.5],
    ["red", "red", 1],
  ])("splits %s", (input, rgb, alpha) => {
    const result = splitAlpha(input);
    expect(result.rgb).toBe(rgb);
    expect(result.alpha).toBeCloseTo(alpha);
  });

  it("caches a material per colour and shape, with the link opacity", () => {
    const gradients = createLinkGradientMaterials(0.8);
    const straight = gradients.material("#ff0000", false);
    expect(gradients.material("#ff0000", false)).toBe(straight);
    const curved = gradients.material("#ff0000", true);
    expect(curved).not.toBe(straight);
    expect(straight.customProgramCacheKey()).not.toBe(
      curved.customProgramCacheKey(),
    );
    expect(straight.opacity).toBeCloseTo(0.8);
    expect(straight.transparent).toBe(true);
    expect(straight.color.getHexString()).toBe("ff0000");

    const dimmed = gradients.material("rgba(255, 0, 0, 0.5)", false);
    expect(dimmed.opacity).toBeCloseTo(0.4);
    expect(dimmed.depthWrite).toBe(false);

    const opaque = createLinkGradientMaterials(1).material("#00ff00", false);
    expect(opaque.transparent).toBe(false);
    expect(opaque.depthWrite).toBe(true);

    gradients.dispose();
    expect(gradients.material("#ff0000", false)).not.toBe(straight);
  });
});
