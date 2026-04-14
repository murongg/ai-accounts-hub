import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

import { zh } from "./zh.ts";
import { en } from "./en.ts";

const shellInstallCommand =
  "curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh";

test("website hero includes npm and shell CLI install commands in both locales", () => {
  for (const [messages, shellLabel] of [
    [zh, "Bash 安装"],
    [en, "Bash install"],
  ] as const) {
    assert.equal(messages.hero.cliInstallCommand, "npm install -g @murongg/aah-cli@latest");
    assert.equal(messages.hero.cliShellInstallLabel, shellLabel);
    assert.equal(messages.hero.cliShellInstallCommand, shellInstallCommand);
    assert.ok(messages.hero.copyCommandLabel);
  }
});

test("website CLI mode documents the shell installer in both locales", () => {
  for (const messages of [zh, en]) {
    const cliMode = messages.modes.items.find((item) => item.title === "aah CLI");

    assert.ok(cliMode, "aah CLI mode should be present");
    assert.equal(cliMode.command, "npm install -g @murongg/aah-cli@latest");
    assert.deepEqual(cliMode.secondaryCommands, [shellInstallCommand]);
  }
});

test("website desktop mode keeps the existing release download copy", () => {
  const zhDesktop = zh.modes.items.find((item) => item.title === "Desktop App");
  const enDesktop = en.modes.items.find((item) => item.title === "Desktop App");

  assert.ok(zhDesktop, "zh desktop mode should be present");
  assert.ok(enDesktop, "en desktop mode should be present");
  assert.equal(zhDesktop.command, "Download from GitHub Releases");
  assert.equal(enDesktop.command, "Download from GitHub Releases");
});

test("website hero renders the shell installer as a separate command row", () => {
  const page = fs.readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");

  assert.match(page, /t\.hero\.cliShellInstallLabel/);
  assert.doesNotMatch(
    page,
    /\[t\.hero\.cliInstallCommand,\s*t\.hero\.cliShellInstallCommand\]\.map/,
  );
});

test("website command rows abbreviate long installs and copy the full command", () => {
  const page = fs.readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");

  assert.match(page, /npm i -g @murongg\/aah-cli@latest/);
  assert.match(page, /curl \.\.\.\/install-aah\.sh \| sh/);
  assert.match(page, /navigator\.clipboard\.writeText\(command\)/);
});

test("website command action buttons are icon-only", () => {
  const page = fs.readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");

  assert.match(page, /aria-label=\{`\$\{copyLabel\}: \$\{command\}`\}/);
  assert.match(page, /<CopyIcon \/>/);
  assert.match(page, /<DownloadIcon \/>/);
  assert.doesNotMatch(page, />\s*\{copyLabel\}\s*<\/button>/);
});

test("website copy buttons show feedback and the hero install block stays compact", () => {
  const page = fs.readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");

  assert.match(page, /<CheckIcon \/>/);
  assert.match(page, /setCopiedKey/);
  assert.match(page, /copied=\{copiedKey === 'hero:cli'\}/);
  assert.match(page, /mt-8 w-fit max-w-full/);
  assert.match(page, /w-full items-center gap-1\.5 rounded-xl/);
  assert.doesNotMatch(page, /sm:grid-cols-\[7rem_minmax\(0,1fr\)\]/);
});
