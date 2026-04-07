import test from "node:test";
import assert from "node:assert/strict";

import { getPlatformAccountMetrics } from "./platformAccountMetrics.ts";

test("returns codex account counts when the codex platform is active", () => {
  const metrics = getPlatformAccountMetrics("codex", [
    { is_active: true },
    { is_active: false },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 3,
    activeCount: 1,
    idleCount: 2,
  });
});

test("returns zero counts for unsupported platforms", () => {
  const metrics = getPlatformAccountMetrics("claude", [
    { is_active: true },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 0,
    activeCount: 0,
    idleCount: 0,
  });
});
