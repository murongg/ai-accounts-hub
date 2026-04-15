import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
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

test("install script supports the standalone aah release installer contract", () => {
  const script = fs.readFileSync(
    new URL("../../../scripts/install-aah.sh", import.meta.url),
    "utf8",
  );

  assert.match(script, /AAH_VERSION/);
  assert.match(script, /AAH_INSTALL_DIR/);
  assert.match(script, /aarch64-apple-darwin/);
  assert.match(script, /x86_64-apple-darwin/);
  assert.match(script, /x86_64-unknown-linux-gnu/);
  assert.match(script, /asset_name="aah_\$\{version\}_\$\{target\}"/);
  assert.match(script, /chmod 755/);
  assert.match(script, /"\$install_dir\/aah" --version/);
});
