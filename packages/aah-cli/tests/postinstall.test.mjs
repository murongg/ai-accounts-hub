import test from "node:test";
import assert from "node:assert/strict";
import {
  assetNameForPlatform,
  binaryPathForPackage,
  releaseTagForVersion,
} from "../scripts/postinstall.mjs";

test("asset name matches darwin arm64 release naming", () => {
  assert.equal(
    assetNameForPlatform("darwin", "arm64", "0.1.0"),
    "aah_0.1.0_aarch64-apple-darwin",
  );
});

test("wrapper resolves the vendored binary path", () => {
  const path = binaryPathForPackage("/tmp/aah-package", "darwin");
  assert.equal(path, "/tmp/aah-package/vendor/aah");
});

test("release tag uses the standalone CLI version namespace", () => {
  assert.equal(releaseTagForVersion("0.1.0"), "cli-v0.1.0");
});
