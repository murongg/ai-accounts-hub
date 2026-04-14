import test from "node:test";
import assert from "node:assert/strict";

import { copyTextToClipboard, normalizeRelayPortInput, relayBaseUrlsFromStatus } from "./relay-settings.ts";

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

test("copyTextToClipboard uses async clipboard when available", async () => {
  let copiedText = "";
  const copied = await copyTextToClipboard("http://127.0.0.1:8765/codex", {
    writeText: async (value) => {
      copiedText = value;
    },
  });

  assert.equal(copied, true);
  assert.equal(copiedText, "http://127.0.0.1:8765/codex");
});

test("copyTextToClipboard falls back to legacy copy when clipboard fails", async () => {
  let copiedText = "";
  const copied = await copyTextToClipboard("http://127.0.0.1:8765/codex", {
    writeText: async () => {
      throw new Error("clipboard unavailable");
    },
    legacyCopy: (value) => {
      copiedText = value;
      return true;
    },
  });

  assert.equal(copied, true);
  assert.equal(copiedText, "http://127.0.0.1:8765/codex");
});

test("copyTextToClipboard returns false when no copy path succeeds", async () => {
  const copied = await copyTextToClipboard("http://127.0.0.1:8765/codex", {
    writeText: async () => {
      throw new Error("clipboard unavailable");
    },
    legacyCopy: () => false,
  });

  assert.equal(copied, false);
});
