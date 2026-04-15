import fs from "node:fs";
import https from "node:https";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPOSITORY = "murongg/ai-accounts-hub";

export function assetNameForPlatform(platform, arch, version) {
  if (platform === "darwin" && arch === "arm64") {
    return `aah_${version}_aarch64-apple-darwin`;
  }
  if (platform === "darwin" && arch === "x64") {
    return `aah_${version}_x86_64-apple-darwin`;
  }
  if (platform === "linux" && arch === "x64") {
    return `aah_${version}_x86_64-unknown-linux-gnu`;
  }
  if (platform === "win32" && arch === "x64") {
    return `aah_${version}_x86_64-pc-windows-msvc.exe`;
  }
  throw new Error(`Unsupported platform ${platform}/${arch}`);
}

export function binaryPathForPackage(packageRoot, platform = process.platform) {
  return platform === "win32"
    ? path.join(packageRoot, "vendor", "aah.exe")
    : path.join(packageRoot, "vendor", "aah");
}

export function releaseTagForVersion(version) {
  return `cli-v${version}`;
}

export function installMetadataPathForEnv(env = process.env, platform = process.platform) {
  if (platform === "win32") {
    const appData = env.APPDATA;
    if (!appData) {
      throw new Error("APPDATA is required on Windows");
    }
    return path.join(appData, "aah", "cli-install.json");
  }

  const home = env.HOME ?? os.homedir();
  const configRoot = env.XDG_CONFIG_HOME ?? path.join(home, ".config");
  return path.join(configRoot, "aah", "cli-install.json");
}

export function detectPackageManagerFromEnv(env = process.env) {
  const userAgent = env.npm_config_user_agent ?? "";
  if (userAgent.startsWith("pnpm/")) {
    return "pnpm";
  }
  if (userAgent.startsWith("yarn/")) {
    return "yarn";
  }
  if (userAgent.startsWith("bun/")) {
    return "bun";
  }
  return "npm";
}

export function writeInstallMetadata(packageRoot, env = process.env, platform = process.platform) {
  const metadataPath = installMetadataPathForEnv(env, platform);
  const binaryPath = binaryPathForPackage(packageRoot, platform);
  const metadata = {
    schema_version: 1,
    install_method: "package-manager",
    package_manager: detectPackageManagerFromEnv(env),
    package_name: "@murongg/aah-cli",
    binary_path: binaryPath,
    installed_at: new Date().toISOString(),
  };

  fs.mkdirSync(path.dirname(metadataPath), { recursive: true });
  fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));
}

function download(url, targetPath) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          download(response.headers.location, targetPath).then(resolve, reject);
          response.resume();
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`Failed to download ${url}: ${response.statusCode}`));
          return;
        }

        const file = fs.createWriteStream(targetPath);
        response.pipe(file);
        file.on("finish", () => file.close(resolve));
        file.on("error", reject);
      })
      .on("error", reject);
  });
}

async function main() {
  const packageRoot = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "..",
  );
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"),
  );
  const assetName = assetNameForPlatform(process.platform, os.arch(), packageJson.version);
  const url = `https://github.com/${REPOSITORY}/releases/download/${releaseTagForVersion(packageJson.version)}/${assetName}`;
  const binaryPath = binaryPathForPackage(packageRoot);

  fs.mkdirSync(path.dirname(binaryPath), { recursive: true });
  await download(url, binaryPath);

  if (process.platform !== "win32") {
    fs.chmodSync(binaryPath, 0o755);
  }

  writeInstallMetadata(packageRoot);
}

if (process.argv[1] && import.meta.url === new URL(process.argv[1], "file:").href) {
  await main();
}
