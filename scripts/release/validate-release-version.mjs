import fs from "node:fs";

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function readCargoPackageVersion(filePath) {
  const source = fs.readFileSync(filePath, "utf8");
  const packageSection = source.match(/\[package\][\s\S]*?(?=\n\[|$)/);

  if (!packageSection) {
    throw new Error(`Could not find [package] section in ${filePath}`);
  }

  const versionMatch = packageSection[0].match(/^\s*version\s*=\s*"([^"]+)"/m);

  if (!versionMatch) {
    throw new Error(`Could not find package version in ${filePath}`);
  }

  return versionMatch[1];
}

function readCargoLockPackageVersion(filePath, packageName) {
  const source = fs.readFileSync(filePath, "utf8");
  const packageBlocks = source.match(/\[\[package\]\][\s\S]*?(?=\n\[\[package\]\]|\n\[metadata\]|\n?$)/g) ?? [];

  for (const block of packageBlocks) {
    const nameMatch = block.match(/^\s*name\s*=\s*"([^"]+)"/m);
    const versionMatch = block.match(/^\s*version\s*=\s*"([^"]+)"/m);

    if (nameMatch?.[1] === packageName && versionMatch?.[1]) {
      return versionMatch[1];
    }
  }

  throw new Error(`Could not find package ${packageName} in ${filePath}`);
}

const releaseTag = process.env.RELEASE_TAG;

if (!releaseTag) {
  throw new Error("RELEASE_TAG is required");
}

if (!/^v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(releaseTag)) {
  throw new Error(`Release tag must look like v1.2.3 or v1.2.3-beta.1, received: ${releaseTag}`);
}

const releaseVersion = releaseTag.replace(/^v/, "");
const packageJson = readJson("package.json");
const packageVersion = packageJson.version;
const tauriVersion = readJson("src-tauri/tauri.conf.json").version;
const cargoVersion = readCargoPackageVersion("src-tauri/Cargo.toml");
const cargoLockVersion = readCargoLockPackageVersion("src-tauri/Cargo.lock", packageJson.name);

const versionSources = {
  "package.json": packageVersion,
  "src-tauri/tauri.conf.json": tauriVersion,
  "src-tauri/Cargo.toml": cargoVersion,
  "src-tauri/Cargo.lock": cargoLockVersion,
};

const mismatches = Object.entries(versionSources).filter(([, version]) => version !== releaseVersion);

if (mismatches.length > 0) {
  const details = mismatches.map(([filePath, version]) => `${filePath}=${version}`).join(", ");
  throw new Error(`Release tag ${releaseTag} does not match project version ${releaseVersion}: ${details}`);
}

console.log(`Release tag ${releaseTag} matches package, Tauri, Cargo, and lockfile versions.`);
