import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

const packageJson = JSON.parse(
  fs.readFileSync(new URL("../package.json", import.meta.url), "utf8"),
);

test("package repository metadata matches GitHub provenance source", () => {
  assert.deepEqual(packageJson.repository, {
    type: "git",
    url: "https://github.com/murongg/ai-accounts-hub",
    directory: "packages/aah-cli",
  });
});
