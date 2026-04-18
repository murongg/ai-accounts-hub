import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const source = readFileSync(path.resolve("src/components/account-list-item.tsx"), "utf8");

test("default list item keeps the wide single-row grid layout", () => {
  assert.match(
    source,
    /: "grid gap-4 p-4 lg:grid-cols-\[minmax\(220px,1\.05fr\)_minmax\(360px,1\.75fr\)_minmax\(150px,0\.75fr\)_auto\] lg:items-center"/,
  );
  assert.match(
    source,
    /const primaryButtonSizeClass = isMini \? "h-8 w-8" : "h-10 min-w-\[128px\] lg:flex-none"/,
  );
  assert.match(source, /if \(!compact\) \{/);
  assert.match(source, /<div className="mb-2 flex items-center justify-between gap-2">/);
  assert.match(source, /className="overflow-hidden rounded-full bg-base-300\/70 h-2"/);
});

test("mini list item keeps a compact horizontal layout", () => {
  assert.match(
    source,
    /const bodyClass = isMini\s+\? "grid gap-3 p-3 sm:grid-cols-\[minmax\(0,1fr\)_minmax\(0,1\.08fr\)_auto\] sm:items-center"\s+:/,
  );
  assert.match(
    source,
    /\{isMini \? \(\s*<MiniStatusIndicators isActive=\{isActive\} isAlive=\{isAlive\} copy=\{copy\} \/>\s*\) : \(/,
  );
  assert.match(source, /aria-label=\{isMini \? primaryActionLabel : undefined\}/);
  assert.match(
    source,
    /\{isMini \? <PrimaryActionIcon isActive=\{isActive\} isBusy=\{primaryDisabled && !isActive\} \/> : primaryActionLabel\}/,
  );
  assert.match(source, /\{isMini \? null : \(\s*<ActivitySummary/);
  assert.match(source, /<div className="min-w-0 grid gap-1\.5">/);
  assert.match(source, /<div className="min-w-0 grid grid-cols-\[34px_minmax\(0,1fr\)_36px\] items-center gap-2\.5">/);
});
