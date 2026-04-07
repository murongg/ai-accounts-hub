import test from "node:test";
import assert from "node:assert/strict";

import { getAccountCardTheme, getQuotaProgressTone } from "./accountCardTheme.ts";

test("returns distinct card styles for active and idle accounts", () => {
  const activeTheme = getAccountCardTheme({ isActive: true, isAlive: true });
  const idleTheme = getAccountCardTheme({ isActive: false, isAlive: true });

  assert.notEqual(activeTheme.cardClass, idleTheme.cardClass);
  assert.notEqual(activeTheme.primaryButtonClass, idleTheme.primaryButtonClass);
});

test("marks unhealthy credentials differently from healthy ones", () => {
  const healthyTheme = getAccountCardTheme({ isActive: false, isAlive: true });
  const unhealthyTheme = getAccountCardTheme({ isActive: false, isAlive: false });

  assert.notEqual(healthyTheme.statusBadgeClass, unhealthyTheme.statusBadgeClass);
  assert.notEqual(healthyTheme.statusDotClass, unhealthyTheme.statusDotClass);
});

test("maps quota tone to remaining percentage severity", () => {
  assert.equal(getQuotaProgressTone(75), "text-emerald-500");
  assert.equal(getQuotaProgressTone(25), "text-warning");
  assert.equal(getQuotaProgressTone(5), "text-error");
});
