#!/usr/bin/env bash

set -euo pipefail

readonly app_name="litestation"
readonly project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly default_appimage_dir="$project_root/target/release/bundle/appimage"
readonly install_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
readonly install_path="$install_dir/$app_name"

usage() {
    cat <<EOF
Install the LiteStation AppImage as '$install_path'.

Usage:
  $(basename "$0") [APPIMAGE]

If APPIMAGE is omitted, the newest AppImage in
$default_appimage_dir is installed.
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
fi

if [[ "${1:-}" == -* ]]; then
    printf 'Unknown option: %s\n\n' "$1" >&2
    usage >&2
    exit 2
fi

if [[ "$(uname -m)" != "x86_64" ]]; then
    printf 'This AppImage is built for x86_64, but this system is %s.\n' "$(uname -m)" >&2
    exit 1
fi

if [[ $# -gt 1 ]]; then
    printf 'Expected zero or one AppImage path.\n\n' >&2
    usage >&2
    exit 2
fi

appimage="${1:-}"
if [[ -z "$appimage" ]]; then
    mapfile -t appimages < <(find "$default_appimage_dir" -maxdepth 1 -type f -name '*.AppImage' -print | sort -V)
    if [[ ${#appimages[@]} -eq 0 ]]; then
        printf 'No AppImage found in %s. Build one with `pnpm tauri build`.\n' "$default_appimage_dir" >&2
        exit 1
    fi
    appimage="${appimages[${#appimages[@]} - 1]}"
fi

if [[ ! -f "$appimage" ]]; then
    printf 'AppImage not found: %s\n' "$appimage" >&2
    exit 1
fi

mkdir -p "$install_dir"
install -m 755 "$appimage" "$install_path"

printf 'Installed LiteStation to %s\n' "$install_path"
if [[ ":${PATH}:" != *":$install_dir:"* ]]; then
    printf 'Add this directory to your PATH to run `litestation` directly:\n'
    printf '  export PATH="%s:$PATH"\n' "$install_dir"
else
    printf 'Run `litestation --help` to get started.\n'
fi
