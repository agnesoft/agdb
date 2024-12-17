import type { TCellType } from "./types";

export const dateFormatter = (value: TCellType): string => {
    return value && typeof value === "number"
        ? new Date(value * 1000).toLocaleString()
        : "N/A";
};
