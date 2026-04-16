import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const appSource = readFileSync(path.resolve("src/App.tsx"), "utf8");

test("accounts view mode updates local state before persistence and rolls back on failure", () => {
  const optimisticIndex = appSource.indexOf("setAppSettings(nextSettings);");
  const persistIndex = appSource.indexOf("const saved = await updateAppSettings(nextSettings);");
  const rollbackIndex = appSource.indexOf("setAppSettings(appSettings);");

  assert.notEqual(optimisticIndex, -1);
  assert.notEqual(persistIndex, -1);
  assert.notEqual(rollbackIndex, -1);
  assert.ok(optimisticIndex < persistIndex);
});
