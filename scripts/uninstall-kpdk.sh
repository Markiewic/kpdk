#!/usr/bin/env bash
set -euo pipefail

yes=false
case "${1:-}" in
  -y|--yes) yes=true ;;
  "") ;;
  *)
    echo "Usage: uninstall-kpdk.sh [--yes]" >&2
    exit 2
    ;;
esac

readonly install_dir="${KPDK_INSTALL_DIR:-$HOME/.local/bin}"
readonly data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
readonly data_dir="$data_home/kpdk"

case "$data_dir" in
  */kpdk) ;;
  *)
    echo "Refusing to remove unexpected data directory: $data_dir" >&2
    exit 1
    ;;
esac

if [[ "$yes" != true ]]; then
  if [[ ! -r /dev/tty || ! -w /dev/tty ]]; then
    echo "No interactive terminal detected. Retry with --yes." >&2
    exit 1
  fi
  printf "Remove kpdk from %s and all SDK data from %s? [y/N] " "$install_dir" "$data_dir" > /dev/tty
  read -r answer < /dev/tty || answer=""
  case "$answer" in
    y|Y|yes|YES|Yes) ;;
    *)
      echo "Uninstall cancelled."
      exit 0
      ;;
  esac
fi

rm -f --   "$install_dir/kpdk"   "$install_dir/easypdkprog"   "$install_dir/easypdkprog-LICENSE"
if [[ -d "$data_dir" ]]; then
  rm -rf -- "$data_dir"
fi

echo "Removed kpdk, Easy PDK Programmer, and all kpdk SDK data."
