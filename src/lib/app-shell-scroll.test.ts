import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("disables app shell overscroll bounce for the main scroll region", () => {
  const appCss = readFileSync("src/App.css", "utf8");
  const appTsx = readFileSync("src/App.tsx", "utf8");
  const mainTsx = readFileSync("src/main.tsx", "utf8");

  assert.match(appCss, /body\s*\{[\s\S]*overflow:\s*hidden;/);
  assert.match(appCss, /#root\s*\{[\s\S]*height:\s*100vh;[\s\S]*overflow:\s*hidden;/);
  assert.match(appCss, /\.app-scroll-region\s*\{[\s\S]*overscroll-behavior-y:\s*none;/);
  assert.match(mainTsx, /className="h-screen overflow-hidden bg-base-200 text-base-content"/);
  assert.match(appTsx, /className="flex h-full min-h-0 w-full flex-col bg-base-200 font-sans text-base-content"/);
  assert.match(appTsx, /className="min-h-0 flex-1 overflow-y-auto app-scroll-region"/);
});
