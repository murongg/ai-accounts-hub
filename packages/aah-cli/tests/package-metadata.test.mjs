import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

const packageJson = JSON.parse(
  fs.readFileSync(new URL("../package.json", import.meta.url), "utf8"),
);

test("package repository metadata matches GitHub provenance source", () => {
  assert.deepEqual(packageJson.repository, {
    type: "git",
    url: "https://github.com/murongg/ai-accounts-hub",
    directory: "packages/aah-cli",
  });
});

test("aah bin entry is executable on Unix-like platforms", () => {
  if (process.platform === "win32") {
    return;
  }

  const binPath = new URL(`../${packageJson.bin.aah}`, import.meta.url);
  const executableBits = fs.statSync(binPath).mode & 0o111;

  assert.notEqual(
    executableBits,
    0,
    `${packageJson.bin.aah} must be executable so global symlinks can run aah`,
  );
});
