import test from "node:test";
import assert from "node:assert/strict";

import { normalizeRelayPortInput, relayBaseUrlsFromStatus } from "./relay-settings.ts";

test("normalizeRelayPortInput accepts valid tcp ports", () => {
  assert.equal(normalizeRelayPortInput("8765"), 8765);
  assert.equal(normalizeRelayPortInput("1"), 1);
  assert.equal(normalizeRelayPortInput("65535"), 65535);
});

test("normalizeRelayPortInput falls back for invalid values", () => {
  assert.equal(normalizeRelayPortInput("", 8765), 8765);
  assert.equal(normalizeRelayPortInput("0", 8765), 8765);
  assert.equal(normalizeRelayPortInput("65536", 8765), 8765);
  assert.equal(normalizeRelayPortInput("abc", 8765), 8765);
});

test("relayBaseUrlsFromStatus preserves backend status urls", () => {
  assert.deepEqual(
    relayBaseUrlsFromStatus({
      running: true,
      bind_host: "127.0.0.1",
      port: 9876,
      last_error: null,
      codex_base_url: "http://127.0.0.1:9876/codex",
    }),
    [["Codex", "http://127.0.0.1:9876/codex"]],
  );
});
