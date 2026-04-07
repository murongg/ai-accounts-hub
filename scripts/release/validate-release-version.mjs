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

const releaseTag = process.env.RELEASE_TAG;

if (!releaseTag) {
  throw new Error("RELEASE_TAG is required");
}

if (!/^v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(releaseTag)) {
  throw new Error(`Release tag must look like v1.2.3 or v1.2.3-beta.1, received: ${releaseTag}`);
}

const releaseVersion = releaseTag.replace(/^v/, "");
const packageVersion = readJson("package.json").version;
const tauriVersion = readJson("src-tauri/tauri.conf.json").version;
const cargoVersion = readCargoPackageVersion("src-tauri/Cargo.toml");

const versionSources = {
  "package.json": packageVersion,
  "src-tauri/tauri.conf.json": tauriVersion,
  "src-tauri/Cargo.toml": cargoVersion,
};

const mismatches = Object.entries(versionSources).filter(([, version]) => version !== releaseVersion);

if (mismatches.length > 0) {
  const details = mismatches.map(([filePath, version]) => `${filePath}=${version}`).join(", ");
  throw new Error(`Release tag ${releaseTag} does not match project version ${releaseVersion}: ${details}`);
}

console.log(`Release tag ${releaseTag} matches package, Tauri, and Cargo versions.`);
