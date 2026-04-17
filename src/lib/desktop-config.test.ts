import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("uses a Windows-safe main window config for phase one", () => {
  const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")) as {
    app: {
      windows: Array<Record<string, unknown>>;
    };
  };

  const mainWindow = config.app.windows[0];

  assert.equal(mainWindow.title, "AI Accounts Hub");
  assert.equal("titleBarStyle" in mainWindow, false);
  assert.notEqual(mainWindow.transparent, true);
});
