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
}

if (process.argv[1] && import.meta.url === new URL(process.argv[1], "file:").href) {
  await main();
}
