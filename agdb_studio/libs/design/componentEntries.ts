import fs from "fs";
import path from "path";

// NOTE: not really needed right now, but might be useful in the future

// This function is used to get all .vue files in the src/ directory
// and return them as an object where the keys are the names of the files
// and the values are the paths to the files.
// This is useful for creating a library of components that can be imported
// and used in other parts of the application.
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
