import { Color, MeshLambertMaterial } from "three";

/** How dark a link is at its source, relative to its colour at the target. */
export const GRADIENT_SOURCE_SHADE = 0.3;

/** Above this many links the gradient is skipped and the default links are drawn. */
export const GRADIENT_MAX_LINKS = 2000;

/** Splits a CSS colour (hex, rgb() or rgba()) into an opaque colour and its alpha. */
export const splitAlpha = (color: string): { rgb: string; alpha: number } => {
  const value = color.trim();
  const hex = /^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(value)?.[1];
  if (hex) {
    const full =
      hex.length <= 4
        ? hex
            .split("")
            .map((c) => c + c)
            .join("")
        : hex;
    const alpha = full.length === 8 ? parseInt(full.slice(6, 8), 16) / 255 : 1;
    return { rgb: `#${full.slice(0, 6)}`, alpha };
  }
  const rgba =
    /^rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)(?:\s*[,/]\s*([\d.]+%?))?\s*\)$/i.exec(
      value,
    );
  if (rgba) {
    const [, r, g, b, a] = rgba;
    const alpha =
      a === undefined
        ? 1
        : a.endsWith("%")
          ? parseFloat(a) / 100
          : parseFloat(a);
    return { rgb: `rgb(${r}, ${g}, ${b})`, alpha };
  }
  return { rgb: value, alpha: 1 };
};

/**
 * Link materials that darken a link towards its source, so its direction
 * reads without selecting it. The library draws straight links as unit
 * cylinders (uv.y runs from the source, 0, to the target, 1) and curved links
 * as tubes (uv.x runs along the curve), so each needs its own variant.
 */
export const createLinkGradientMaterials = (opacity: number) => {
  const cache = new Map<string, MeshLambertMaterial>();

  const material = (color: string, curved: boolean): MeshLambertMaterial => {
    const key = `${curved ? "c" : "s"}|${color}`;
    const cached = cache.get(key);
    if (cached) return cached;
    const { rgb, alpha } = splitAlpha(color);
    const finalOpacity = opacity * alpha;
    const created = new MeshLambertMaterial({
      color: new Color(rgb),
      transparent: finalOpacity < 1,
      opacity: finalOpacity,
      depthWrite: finalOpacity >= 1,
    });
    const along = curved ? "uv.x" : "uv.y";
    /* v8 ignore start -- @preserve: runs when WebGL compiles the shader */
    created.onBeforeCompile = (shader) => {
      shader.vertexShader = shader.vertexShader
        .replace("void main() {", "varying float vLinkT;\nvoid main() {")
        .replace(
          "#include <begin_vertex>",
          `#include <begin_vertex>\nvLinkT = ${along};`,
        );
      shader.fragmentShader = shader.fragmentShader
        .replace("void main() {", "varying float vLinkT;\nvoid main() {")
        .replace(
          "#include <color_fragment>",
          `#include <color_fragment>\ndiffuseColor.rgb *= mix(${GRADIENT_SOURCE_SHADE.toFixed(2)}, 1.0, vLinkT);`,
        );
    };
    /* v8 ignore stop -- @preserve */
    created.customProgramCacheKey = () => `link-gradient-${along}`;
    cache.set(key, created);
    return created;
  };

  const dispose = () => {
    for (const m of cache.values()) m.dispose();
    cache.clear();
  };

  return { material, dispose };
};
