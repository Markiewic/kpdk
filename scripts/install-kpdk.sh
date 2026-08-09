#!/usr/bin/env bash
set -euo pipefail

readonly repository="Markiewic/kpdk"
readonly install_dir="${KPDK_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
  Linux) platform="linux" ;;
  *)
    echo "The one-command installer currently supports Linux. Use the release archive on this platform." >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64) architecture="x64" ;;
  aarch64|arm64) architecture="arm64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

archive="kpdk-${platform}-${architecture}.tar.gz"
base_url="https://github.com/${repository}/releases/latest/download"
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

echo "Downloading $archive..."
curl --fail --location --silent --show-error "$base_url/$archive" --output "$work_dir/$archive"
curl --fail --location --silent --show-error "$base_url/$archive.sha256" --output "$work_dir/$archive.sha256"
(
  cd "$work_dir"
  sha256sum --check "$archive.sha256"
)

mkdir -p "$install_dir"
tar -xzf "$work_dir/$archive" -C "$install_dir"
chmod 755 "$install_dir/kpdk" "$install_dir/easypdkprog"

profile="$HOME/.profile"
path_line='export PATH="$HOME/.local/bin:$PATH"'
if [[ "$install_dir" == "$HOME/.local/bin" ]] && [[ ":$PATH:" != *":$install_dir:"* ]]; then
  if ! grep -Fqx "$path_line" "$profile" 2>/dev/null; then
    {
      echo
      echo "# Added by the kpdk installer"
      echo "$path_line"
    } >> "$profile"
  fi
fi

"$install_dir/kpdk" --version
"$install_dir/easypdkprog" --version

echo "Installed kpdk and easypdkprog in $install_dir"
if [[ ":$PATH:" != *":$install_dir:"* ]]; then
  echo "Restart your shell or run: export PATH=\"$install_dir:\$PATH\""
fi
if [[ -r /dev/tty && -w /dev/tty ]]; then
  printf "The SDK is required to build Padauk projects. Install it now? [Y/n] " > /dev/tty
  if ! read -r install_sdk < /dev/tty; then
    install_sdk=""
  fi
  case "$install_sdk" in
    ""|y|Y|yes|YES|Yes)
      "$install_dir/kpdk" sdk install
      ;;
    *)
      echo "SDK installation skipped. Install it before your first build with: kpdk sdk install"
      ;;
  esac
else
  echo "No interactive terminal detected. Install the SDK before your first build with: kpdk sdk install"
fi
