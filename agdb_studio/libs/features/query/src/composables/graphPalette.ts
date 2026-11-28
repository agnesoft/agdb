import { onMounted, onUnmounted, shallowRef } from "vue";
import {
  GRAPH_THEME_FALLBACK,
  readGraphTheme,
  type GraphTheme,
} from "../utils/graphStyle";

/** Colours from the CSS theme, kept up to date with the OS colour scheme. */
const useThemeColors = <T>(pick: (theme: GraphTheme) => T) => {
  const colors = shallowRef<T>(pick(GRAPH_THEME_FALLBACK));
  const read = () => {
    colors.value = pick(readGraphTheme(document.documentElement));
  };
  let query: MediaQueryList | undefined;
  onMounted(() => {
    read();
    query = window.matchMedia?.("(prefers-color-scheme: dark)");
    query?.addEventListener?.("change", read);
  });
  onUnmounted(() => query?.removeEventListener?.("change", read));
  return colors;
};

/**
 * The graph's categorical palette, for controls outside the graph (e.g. row
 * swatches).
 */
export const useGraphPalette = () => useThemeColors((theme) => theme.palette);

/** The graph's gradient scales, each low to high, for legends and previews. */
export const useGraphRamps = () => useThemeColors((theme) => theme.ramps);
