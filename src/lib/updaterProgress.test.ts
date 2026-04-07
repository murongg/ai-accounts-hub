import test from "node:test";
import assert from "node:assert/strict";

import { formatUpdateProgress } from "./updaterProgress.ts";

test("formats update download progress with total size", () => {
  assert.equal(
    formatUpdateProgress({ downloadedBytes: 1_572_864, totalBytes: 5_242_880 }, "zh-CN"),
    "1.5 MB / 5.0 MB",
  );
});

test("formats update download progress without total size", () => {
  assert.equal(
    formatUpdateProgress({ downloadedBytes: 1_572_864 }, "en-US"),
    "1.5 MB downloaded",
  );
});
