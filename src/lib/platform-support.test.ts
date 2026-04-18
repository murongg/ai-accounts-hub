import test from "node:test";
import assert from "node:assert/strict";

import {
  getAccountsActionSupport,
  normalizeDesktopPlatform,
} from "./platform-support.ts";

test("normalizes common desktop platform labels", () => {
  assert.equal(normalizeDesktopPlatform("Windows_NT"), "windows");
  assert.equal(normalizeDesktopPlatform("darwin"), "macos");
  assert.equal(normalizeDesktopPlatform("linux"), "linux");
});

test("keeps account actions enabled on Windows", () => {
  const support = getAccountsActionSupport({
    platform: "windows",
    language: "zh-CN",
  });

  assert.equal(support.actionsEnabled, true);
  assert.equal(support.badge, null);
  assert.equal(support.reason, null);
});
