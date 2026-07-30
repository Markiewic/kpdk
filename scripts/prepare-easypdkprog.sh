#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <destination>" >&2
  exit 2
fi

readonly revision="c704defbf90a934d5d4969bded63e824048e0d24"
readonly expected_sha256="3ae7b41224afbd35029257eceba1bc636ec2f87eb511a1545ad7d812db3e5b5c"
readonly source_url="https://codeload.github.com/free-pdk/easy-pdk-programmer-software/tar.gz/$revision"
readonly destination="$1"
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

curl --fail --location --silent --show-error "$source_url" --output "$work_dir/source.tar.gz"
echo "$expected_sha256  $work_dir/source.tar.gz" | sha256sum --check -
mkdir "$work_dir/source"
tar -xzf "$work_dir/source.tar.gz" -C "$work_dir/source" --strip-components=1
make -C "$work_dir/source"
mkdir -p "$(dirname "$destination")"
install -m 755 "$work_dir/source/easypdkprog" "$destination"
install -m 644 "$work_dir/source/LICENSE" "$(dirname "$destination")/easypdkprog-LICENSE"
"$destination" --version
