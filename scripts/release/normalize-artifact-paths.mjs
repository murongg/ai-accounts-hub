import fs from "node:fs";
import path from "node:path";

function addArtifactPath(artifactPaths, artifactPath) {
  if (!artifactPath) {
    return;
  }

  if (artifactPath.endsWith("/latest.json") || artifactPath.endsWith("\\latest.json")) {
    return;
  }

  try {
    if (!fs.statSync(artifactPath).isFile()) {
      return;
    }
  } catch {
    return;
  }

  if (!artifactPaths.includes(artifactPath)) {
    artifactPaths.push(artifactPath);
  }
}

function walkFiles(rootDir) {
  const stack = [rootDir];
  const files = [];

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

      if (entry.isFile()) {
        files.push(fullPath);
      }
    }
  }

  return files;
}

const customDmgPath = process.env.CUSTOM_DMG_PATH || "";
const bundleRoot = process.env.BUNDLE_ROOT || "";
const artifactPaths = [];

for (const artifactPath of JSON.parse(process.env.ARTIFACT_PATHS_JSON || "[]")) {
  if (customDmgPath && artifactPath.endsWith(".dmg")) {
    continue;
  }

  addArtifactPath(artifactPaths, artifactPath);
}

for (const bundleFile of walkFiles(bundleRoot)) {
  if (!bundleFile.endsWith(".sig")) {
    continue;
  }

  addArtifactPath(artifactPaths, bundleFile);
  addArtifactPath(artifactPaths, bundleFile.slice(0, -4));
}

if (customDmgPath) {
  addArtifactPath(artifactPaths, customDmgPath);
}

fs.appendFileSync(
  process.env.GITHUB_OUTPUT,
  `paths<<__PATHS__\n${artifactPaths.join("\n")}\n__PATHS__\n`,
);
