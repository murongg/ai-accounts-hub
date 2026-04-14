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

if (!/^cli-v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(releaseTag)) {
  throw new Error(`CLI release tag must look like cli-v1.2.3 or cli-v1.2.3-beta.1, received: ${releaseTag}`);
}

const releaseVersion = releaseTag.replace(/^cli-v/, "");
const npmPackageVersion = readJson("packages/aah-cli/package.json").version;
const cargoVersion = readCargoPackageVersion("crates/aah-cli/Cargo.toml");
const cargoLockVersion = readCargoLockPackageVersion("Cargo.lock", "aah-cli");

const versionSources = {
  "packages/aah-cli/package.json": npmPackageVersion,
  "crates/aah-cli/Cargo.toml": cargoVersion,
  "Cargo.lock aah-cli": cargoLockVersion,
};

const mismatches = Object.entries(versionSources).filter(([, version]) => version !== releaseVersion);

if (mismatches.length > 0) {
  const details = mismatches.map(([filePath, version]) => `${filePath}=${version}`).join(", ");
  throw new Error(`CLI release tag ${releaseTag} does not match CLI version ${releaseVersion}: ${details}`);
}

console.log(`CLI release tag ${releaseTag} matches CLI versions.`);
