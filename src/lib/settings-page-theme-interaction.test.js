import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const source = readFileSync(path.resolve("src/pages/settings-page.tsx"), "utf8");

test("theme buttons stay interactive while app settings are saving", () => {
  assert.ok(!source.includes('disabled={isSavingAppSettings}\n                      onClick={() => onThemeChange("light")}'));
  assert.ok(!source.includes('disabled={isSavingAppSettings}\n                      onClick={() => onThemeChange("dark")}'));
  assert.ok(!source.includes('disabled={isSavingAppSettings}\n                      onClick={() => onThemeChange("system")}'));
});
