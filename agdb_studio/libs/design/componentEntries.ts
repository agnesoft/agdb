import fs from "fs";
import path from "path";

// Recursively get all .vue files in src/
export const getVueComponentEntries = (
  dir: string,
  root = dir,
): Record<string, string> => {
  const entries: Record<string, string> = {};

  for (const file of fs.readdirSync(dir)) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory()) {
      Object.assign(entries, getVueComponentEntries(fullPath, root));
    } else if (file.endsWith(".vue")) {
      const relativePath = path.relative(root, fullPath);
      const name = relativePath
        .replace(/\.vue$/, "") // remove extension
        .replace(/[\/\\]/g, "_"); // convert nested path to flat name
      entries[name] = fullPath;
    }
  }

  return entries;
};
