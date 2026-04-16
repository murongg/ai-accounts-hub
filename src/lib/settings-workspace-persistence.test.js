import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const source = readFileSync(path.resolve("src/containers/settings-workspace.tsx"), "utf8");

test("settings workspace updates local app settings before persistence and rolls back on failure", () => {
  const optimisticIndex = source.indexOf("onAppSettingsChange(nextSettings);");
  const persistIndex = source.indexOf("const saved = await updateAppSettings(nextSettings);");
  const rollbackIndex = source.indexOf("onAppSettingsChange(previousSettings);");

  assert.notEqual(optimisticIndex, -1);
  assert.notEqual(persistIndex, -1);
  assert.notEqual(rollbackIndex, -1);
  assert.ok(optimisticIndex < persistIndex);
});
