import test from "node:test";
import assert from "node:assert/strict";

import {
  ACCOUNTS_VIEW_MODES,
  getAccountsViewModeIconName,
  normalizeAccountsViewMode,
} from "./accounts-view-mode.ts";

test("normalizeAccountsViewMode falls back to cards for unknown values", () => {
  assert.equal(normalizeAccountsViewMode("cards"), "cards");
  assert.equal(normalizeAccountsViewMode("list"), "list");
  assert.equal(normalizeAccountsViewMode("table"), "cards");
  assert.equal(normalizeAccountsViewMode(undefined), "cards");
});

test("accounts view modes stay in card-first order", () => {
  assert.deepEqual(ACCOUNTS_VIEW_MODES, ["cards", "list"]);
});

test("accounts view modes use distinct icon names", () => {
  assert.equal(getAccountsViewModeIconName("cards"), "layout-grid");
  assert.equal(getAccountsViewModeIconName("list"), "layout-list");
});
