import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = fileURLToPath(new URL("../..", import.meta.url));

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "aah-updater-manifest-"));
}

function writeArtifact(rootDir, artifactName, signatureName) {
  const artifactDir = path.join(rootDir, "darwin-aarch64");
  fs.mkdirSync(artifactDir, { recursive: true });
  fs.writeFileSync(path.join(artifactDir, artifactName), "bundle");
  fs.writeFileSync(path.join(artifactDir, signatureName), "signature");
  fs.writeFileSync(
    path.join(artifactDir, "release-metadata.json"),
    JSON.stringify(
      {
        updaterPlatform: "darwin-aarch64",
        bundleName: artifactName,
        signatureName,
      },
      null,
      2,
    ),
  );
}

function runManifestScript(env) {
  return spawnSync(process.execPath, ["scripts/release/generate-updater-manifest.mjs"], {
    cwd: repoRoot,
    env: {
      ...process.env,
      ...env,
    },
    encoding: "utf8",
  });
}

test("generate-updater-manifest publishes updater URLs through the latest release endpoint", () => {
  const rootDir = makeTempDir();
  const notesPath = path.join(rootDir, "release-notes.md");
  const artifactName = "ai-accounts-hub_0.3.20_aarch64.app.tar.gz";
  const signatureName = `${artifactName}.sig`;

  writeArtifact(rootDir, artifactName, signatureName);
  fs.writeFileSync(notesPath, "Release notes");

  const result = runManifestScript({
    GITHUB_REPOSITORY: "murongg/ai-accounts-hub",
    RELEASE_ASSETS_ROOT: rootDir,
    RELEASE_NOTES_PATH: notesPath,
    RELEASE_VERSION: "0.3.20",
  });

  assert.equal(result.status, 0, result.stderr);

  const manifest = JSON.parse(fs.readFileSync(path.join(rootDir, "latest.json"), "utf8"));
  assert.equal(
    manifest.platforms["darwin-aarch64"].url,
    `https://github.com/murongg/ai-accounts-hub/releases/latest/download/${artifactName}`,
  );
});
