// Extract the map variant table from the pinned OverPy oracle.
// Usage: node extract.mjs <path-to-overpy.js> > ../../crates/workshop-rs/tests/fixtures/map-variants/overpy-9.7.17.json
import { readFileSync } from "node:fs";

const lines = readFileSync(process.argv[2], "utf8").split("\n");
const start = lines.findIndex((line) => line.startsWith("var mapKw = ")) + 2;
const end = lines.findIndex((line, i) => i > start && line === "  }");
const mapKw = (0, eval)(`(${lines.slice(start, end + 1).join("\n")})`);
const rows = Object.entries(mapKw)
  .filter(([, map]) => map.variants)
  .map(([key, map]) => ({ key, enUS: map["en-US"], variants: map.variants }));
process.stdout.write(JSON.stringify(rows, null, 2) + "\n");
