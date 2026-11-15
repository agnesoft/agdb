import { onMounted, onUnmounted, ref } from "vue";
import { GRAPH_THEME_FALLBACK, readGraphTheme } from "../utils/graphStyle";

/**
 * The graph's categorical palette from the CSS theme, kept up to date with
 * the OS colour scheme. For controls outside the graph (e.g. row swatches).
 */
export const useGraphPalette = () => {
  const palette = ref<string[]>([...GRAPH_THEME_FALLBACK.palette]);
  const read = () => {
    palette.value = readGraphTheme(document.documentElement).palette;
  };
  let query: MediaQueryList | undefined;
  onMounted(() => {
    read();
    query = window.matchMedia?.("(prefers-color-scheme: dark)");
    query?.addEventListener?.("change", read);
  });
  onUnmounted(() => query?.removeEventListener?.("change", read));
  return palette;
};
