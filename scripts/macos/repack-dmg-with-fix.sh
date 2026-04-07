#!/bin/bash

set -euo pipefail

if [[ $# -lt 2 || $# -gt 4 ]]; then
  echo "Usage: $0 <app-path> <output-dmg> [volume-name] [fix-script]"
  exit 1
fi

APP_PATH="$1"
OUTPUT_DMG="$2"
VOLUME_NAME="${3:-ai-accounts-hub}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIX_SCRIPT="${4:-${SCRIPT_DIR}/fix-ai-accounts-hub.command}"
FIX_SCRIPT_NAME="Fix AI Accounts Hub.command"

if [[ ! -d "${APP_PATH}" ]]; then
  echo "App bundle not found: ${APP_PATH}"
  exit 1
fi

if [[ ! -f "${FIX_SCRIPT}" ]]; then
  echo "Fix script not found: ${FIX_SCRIPT}"
  exit 1
fi

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ai-accounts-hub-dmg.XXXXXX")"
STAGE_DIR="${TMP_ROOT}/stage"

cleanup() {
  rm -rf "${TMP_ROOT}"
}

trap cleanup EXIT

mkdir -p "${STAGE_DIR}"
/usr/bin/ditto "${APP_PATH}" "${STAGE_DIR}/$(basename "${APP_PATH}")"
/bin/cp "${FIX_SCRIPT}" "${STAGE_DIR}/${FIX_SCRIPT_NAME}"
/bin/chmod +x "${STAGE_DIR}/${FIX_SCRIPT_NAME}"
/bin/ln -s /Applications "${STAGE_DIR}/Applications"

/bin/mkdir -p "$(dirname "${OUTPUT_DMG}")"
/bin/rm -f "${OUTPUT_DMG}"

/usr/bin/hdiutil create \
  -volname "${VOLUME_NAME}" \
  -srcfolder "${STAGE_DIR}" \
  -ov \
  -format UDZO \
  "${OUTPUT_DMG}"

echo "Created ${OUTPUT_DMG}"
