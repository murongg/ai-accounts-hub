#!/bin/bash

set -euo pipefail

APP_NAME="AI Accounts Hub.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SOURCE_APP="${SCRIPT_DIR}/${APP_NAME}"
SYSTEM_TARGET="/Applications/${APP_NAME}"
USER_TARGET="${HOME}/Applications/${APP_NAME}"

print_line() {
  printf '%s\n' "$1"
}

run_admin_command() {
  /usr/bin/osascript - "$1" <<'OSA'
on run argv
  do shell script (item 1 of argv) with administrator privileges
end run
OSA
}

repair_app() {
  local app_path="$1"
  local command

  print_line ""
  print_line "Removing macOS quarantine attribute from:"
  print_line "  ${app_path}"

  if [[ "${app_path}" == /Applications/* ]]; then
    printf -v command '/usr/bin/xattr -dr com.apple.quarantine %q >/dev/null 2>&1 || true' "${app_path}"
    run_admin_command "${command}"
  else
    /usr/bin/xattr -dr com.apple.quarantine "${app_path}" >/dev/null 2>&1 || true
  fi

  print_line "Quarantine attribute removed."
}

install_from_dmg() {
  local command

  if [[ ! -d "${SOURCE_APP}" ]]; then
    print_line "Could not find ${APP_NAME} next to this script."
    exit 1
  fi

  print_line ""
  print_line "Installing ${APP_NAME} to /Applications ..."
  printf -v command '/usr/bin/ditto %q %q' "${SOURCE_APP}" "${SYSTEM_TARGET}"
  run_admin_command "${command}"
  repair_app "${SYSTEM_TARGET}"
}

resolve_target_app() {
  if [[ $# -gt 0 && -d "$1" ]]; then
    printf '%s\n' "$1"
    return
  fi

  if [[ -d "${SYSTEM_TARGET}" ]]; then
    printf '%s\n' "${SYSTEM_TARGET}"
    return
  fi

  if [[ -d "${USER_TARGET}" ]]; then
    printf '%s\n' "${USER_TARGET}"
    return
  fi

  printf '%s\n' ""
}

main() {
  clear || true
  print_line "AI Accounts Hub macOS repair helper"
  print_line "=================================="

  local target_app
  target_app="$(resolve_target_app "${1:-}")"

  if [[ -z "${target_app}" ]]; then
    install_from_dmg
    target_app="${SYSTEM_TARGET}"
  else
    repair_app "${target_app}"
  fi

  print_line ""
  print_line "Opening ${APP_NAME} ..."
  /usr/bin/open "${target_app}"
  print_line "Done."
}

main "$@"
