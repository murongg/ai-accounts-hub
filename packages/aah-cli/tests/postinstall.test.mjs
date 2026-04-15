import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import {
  assetNameForPlatform,
  binaryPathForPackage,
  detectPackageManagerFromEnv,
  installMetadataPathForEnv,
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

test("postinstall stores install metadata under the user config dir", () => {
  assert.equal(
    installMetadataPathForEnv({
      HOME: "/Users/example",
      XDG_CONFIG_HOME: "/Users/example/.config",
    }),
    "/Users/example/.config/aah/cli-install.json",
  );
});

test("postinstall detects pnpm from npm_config_user_agent", () => {
  assert.equal(
    detectPackageManagerFromEnv({
      npm_config_user_agent: "pnpm/10.0.0 npm/? node/v22.0.0 darwin arm64",
    }),
    "pnpm",
  );
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
  assert.match(script, /cli-install\.json/);
  assert.match(script, /"install_method": "binary"/);
  assert.match(script, /chmod 755/);
  assert.match(script, /"\$install_dir\/aah" --version/);
});

test("CLI docs mention the upgrade command", () => {
  const packageReadme = fs.readFileSync(
    new URL("../README.md", import.meta.url),
    "utf8",
  );

  assert.match(packageReadme, /aah upgrade/);
});
