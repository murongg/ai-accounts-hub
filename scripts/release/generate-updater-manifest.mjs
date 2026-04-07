import fs from "node:fs";
import path from "node:path";

function findFileByBasename(rootDir, fileName) {
  const stack = [rootDir];

  while (stack.length > 0) {
    const currentDir = stack.pop();
    if (!currentDir || !fs.existsSync(currentDir)) {
      continue;
    }

    for (const entry of fs.readdirSync(currentDir, { withFileTypes: true })) {
      const fullPath = path.join(currentDir, entry.name);

      if (entry.isDirectory()) {
        stack.push(fullPath);
        continue;
      }

      if (entry.isFile() && entry.name === fileName) {
        return fullPath;
      }
    }
  }

  return null;
}

const root = process.env.RELEASE_ASSETS_ROOT || "release-assets";
const version = process.env.RELEASE_VERSION;
const notesPath = process.env.RELEASE_NOTES_PATH || "release-notes.md";
const repository = process.env.GITHUB_REPOSITORY;
const notes = fs.readFileSync(notesPath, "utf8").trim();
const artifactDirs = fs
  .readdirSync(root, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => path.join(root, entry.name));

const platforms = {};

for (const artifactDir of artifactDirs) {
  const metadataPath = path.join(artifactDir, "release-metadata.json");
  if (!fs.existsSync(metadataPath)) {
    continue;
  }

  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
  const bundlePath = findFileByBasename(artifactDir, metadata.bundleName);
  const signaturePath = findFileByBasename(artifactDir, metadata.signatureName);

  if (!bundlePath || !fs.existsSync(bundlePath)) {
    throw new Error(`Missing updater bundle: ${metadata.bundleName} in ${artifactDir}`);
  }

  if (!signaturePath || !fs.existsSync(signaturePath)) {
    throw new Error(`Missing updater signature: ${metadata.signatureName} in ${artifactDir}`);
  }

  platforms[metadata.updaterPlatform] = {
    signature: fs.readFileSync(signaturePath, "utf8").trim(),
    url: `https://github.com/${repository}/releases/latest/download/${metadata.bundleName}`,
  };
}

if (Object.keys(platforms).length === 0) {
  throw new Error("No updater platforms were collected from workflow artifacts.");
}

const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms,
};

fs.writeFileSync(path.join(root, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
