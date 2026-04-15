#!/bin/sh
set -eu

repo="${AAH_REPOSITORY:-murongg/ai-accounts-hub}"

die() {
  printf 'aah installer: %s\n' "$*" >&2
  exit 1
}

has() {
  command -v "$1" >/dev/null 2>&1
}

fetch_text() {
  url="$1"

  if has curl; then
    curl -fsSL "$url"
    return
  fi

  if has wget; then
    wget -qO- "$url"
    return
  fi

  die "curl or wget is required"
}

download_file() {
  url="$1"
  output="$2"

  if has curl; then
    curl -fsSL "$url" -o "$output"
    return
  fi

  if has wget; then
    wget -q "$url" -O "$output"
    return
  fi

  die "curl or wget is required"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Darwin:arm64 | Darwin:aarch64)
      printf '%s\n' "aarch64-apple-darwin"
      ;;
    Darwin:x86_64 | Darwin:amd64)
      printf '%s\n' "x86_64-apple-darwin"
      ;;
    Linux:x86_64 | Linux:amd64)
      printf '%s\n' "x86_64-unknown-linux-gnu"
      ;;
    *)
      die "unsupported platform $os/$arch; supported targets are macOS arm64, macOS x64, and Linux x64"
      ;;
  esac
}

latest_cli_version() {
  releases_url="https://api.github.com/repos/$repo/releases?per_page=100"
  releases="$(fetch_text "$releases_url")" || die "failed to fetch releases for $repo"
  tag="$(
    printf '%s\n' "$releases" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\(cli-v[0-9][^"]*\)".*/\1/p' |
      head -n 1
  )"

  if [ -z "$tag" ]; then
    die "could not find a cli-vX.Y.Z release for $repo"
  fi

  printf '%s\n' "${tag#cli-v}"
}

version="${AAH_VERSION:-}"
if [ -z "$version" ]; then
  version="$(latest_cli_version)"
fi
version="${version#cli-v}"

if [ -z "$version" ]; then
  die "AAH_VERSION resolved to an empty version"
fi

if [ -n "${AAH_INSTALL_DIR:-}" ]; then
  install_dir="$AAH_INSTALL_DIR"
else
  if [ -z "${HOME:-}" ]; then
    die "HOME is required when AAH_INSTALL_DIR is not set"
  fi
  install_dir="$HOME/.local/bin"
fi

if [ -n "${XDG_CONFIG_HOME:-}" ]; then
  metadata_dir="$XDG_CONFIG_HOME/aah"
elif [ -n "${HOME:-}" ]; then
  metadata_dir="$HOME/.config/aah"
else
  metadata_dir=""
fi
metadata_path="${metadata_dir:+$metadata_dir/cli-install.json}"

target="$(detect_target)"
asset_name="aah_${version}_${target}"
release_tag="cli-v${version}"
download_url="https://github.com/$repo/releases/download/$release_tag/$asset_name"

tmp_dir="${TMPDIR:-/tmp}/aah-install.$$"
tmp_bin="$tmp_dir/aah"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT HUP INT TERM

write_install_metadata() {
  if [ -z "$metadata_path" ]; then
    return
  fi

  mkdir -p "$metadata_dir" || die "failed to create metadata directory: $metadata_dir"
  cat >"$metadata_path" <<EOF
{
  "schema_version": 1,
  "install_method": "binary",
  "binary_path": "$install_dir/aah",
  "install_dir": "$install_dir",
  "repository": "$repo"
}
EOF
}

mkdir -p "$tmp_dir"

printf 'Installing aah %s for %s...\n' "$version" "$target"
download_file "$download_url" "$tmp_bin"

mkdir -p "$install_dir" || die "failed to create install directory: $install_dir"
cp "$tmp_bin" "$install_dir/aah" || die "failed to copy aah into $install_dir"
chmod 755 "$install_dir/aah" || die "failed to make $install_dir/aah executable"
write_install_metadata

if "$install_dir/aah" --version >/dev/null 2>&1; then
  :
else
  "$install_dir/aah" --help >/dev/null 2>&1 || die "installed binary could not be executed"
fi

printf 'aah installed to %s\n' "$install_dir/aah"

case ":$PATH:" in
  *":$install_dir:"*)
    ;;
  *)
    printf 'Add %s to your PATH to run aah from any shell.\n' "$install_dir"
    ;;
esac
