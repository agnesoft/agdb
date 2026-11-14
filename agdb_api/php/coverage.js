// Fails unless the PHPUnit text coverage summary reports 100 % classes, methods and lines.
import { readFileSync } from "node:fs";

const report = readFileSync("coverage/coverage.txt", "utf8");
console.log(report);
const summary = report.slice(report.indexOf("Summary:"));
const insufficient = ["Classes", "Methods", "Lines"].filter((metric) => !new RegExp(`${metric}:\\s+100\\.00%`).test(summary));

if (insufficient.length > 0) {
    console.error(`Insufficient coverage: ${insufficient.join(", ")}`);
    process.exit(1);
}
