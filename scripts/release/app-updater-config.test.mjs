import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

const tauriConfig = JSON.parse(
  fs.readFileSync(new URL("../../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
);

test("tauri updater keeps the legacy latest.json endpoint for compatibility", () => {
  assert.deepEqual(tauriConfig.plugins.updater.endpoints, [
    "https://github.com/murongg/ai-accounts-hub/releases/latest/download/latest.json",
  ]);
});
