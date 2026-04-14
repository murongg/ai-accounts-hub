import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = fileURLToPath(new URL("../..", import.meta.url));

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(new URL(`../../${filePath}`, import.meta.url), "utf8"));
}

function runScript(scriptPath, releaseTag) {
  return spawnSync(process.execPath, [scriptPath], {
    cwd: repoRoot,
    env: {
      ...process.env,
      RELEASE_TAG: releaseTag,
    },
    encoding: "utf8",
  });
}

test("app release version validation is scoped to the app version", () => {
  const appVersion = readJson("package.json").version;
  const result = runScript("scripts/release/validate-release-version.mjs", `v${appVersion}`);

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /matches app versions/);
});

test("cli release version validation uses the standalone CLI tag namespace", () => {
  const cliVersion = readJson("packages/aah-cli/package.json").version;
  const result = runScript("scripts/release/validate-cli-release-version.mjs", `cli-v${cliVersion}`);

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /matches CLI versions/);
});
