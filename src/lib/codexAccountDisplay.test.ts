import test from "node:test";
import assert from "node:assert/strict";

import { formatResetLabel, formatTimestamp } from "./codexAccountDisplay.ts";

test("formats relative sync timestamps in English when requested", () => {
  const now = Date.now;
  Date.now = () => 1_700_000_000_000;

  try {
    assert.equal(formatTimestamp(String(1_700_000_000 - (15 * 60)), "Just now", "en-US"), "15m ago");
    assert.equal(formatTimestamp(String(1_700_000_000 - (4 * 60 * 60)), "Just now", "en-US"), "4h ago");
  } finally {
    Date.now = now;
  }
});

test("formats reset labels in English when requested", () => {
  assert.equal(formatResetLabel("--:--", "en-US"), "Reset time --:--");
  assert.equal(formatResetLabel("Refresh soon", "en-US"), "Resets soon");
  assert.equal(formatResetLabel("4h 35m", "en-US"), "Resets in 4h 35m");
});
