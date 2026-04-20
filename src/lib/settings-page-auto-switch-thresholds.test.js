import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const source = readFileSync(path.resolve("src/pages/settings-page.tsx"), "utf8");

test("auto switch threshold inputs stay numeric and disable while auto switch is enabled", () => {
  assert.ok(source.includes('inputMode="numeric"'));
  assert.ok(source.includes("maxLength={2}"));
  assert.ok(source.includes("disabled={autoSwitchEnabled}"));
  assert.ok(source.includes("copy.settings.autoSwitch.fiveHourThresholdLabel"));
  assert.ok(source.includes("copy.settings.autoSwitch.weeklyThresholdLabel"));
});
