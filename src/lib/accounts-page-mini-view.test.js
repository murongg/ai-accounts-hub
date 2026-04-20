import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const source = readFileSync(path.resolve("src/pages/accounts-page.tsx"), "utf8");

test("mini view reuses the list layout with the compact list item variant", () => {
  assert.match(source, /\) : viewMode === "list" \|\| viewMode === "mini" \? \(/);
  assert.match(source, /viewMode === "mini" \? "gap-2 md:grid-cols-2 xl:grid-cols-3" : "gap-3"/);
  assert.match(source, /variant=\{viewMode === "mini" \? "mini" : "default"\}/);
});
