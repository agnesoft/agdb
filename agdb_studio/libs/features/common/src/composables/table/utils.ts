import type { TCellType } from "./types";

export const dateFormatter = (value: TCellType): string => {
  return value && typeof value === "number"
    ? new Date(value * 1000).toUTCString()
    : "N/A";
};

export const numberFormatter = (value: TCellType): string => {
  return typeof value === "number" ? value.toLocaleString("en-US") : "N/A";
};
