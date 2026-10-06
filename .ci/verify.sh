#!/usr/bin/env bash
set -euo pipefail
vale_dir="${RUNNER_TEMP:-/tmp}/vale"
mkdir -p "$vale_dir"
curl --fail --location --silent --show-error \
  https://github.com/vale-cli/vale/releases/download/v3.23.0/vale_3.23.0_Linux_64-bit.tar.gz \
  -o "${RUNNER_TEMP:-/tmp}/vale.tar.gz"
echo "cc35445a45186b8f0b01e11c01359694cf941e72cf6ab0fc44774f0e54c9d5fc  ${RUNNER_TEMP:-/tmp}/vale.tar.gz" | sha256sum --check
tar -xzf "${RUNNER_TEMP:-/tmp}/vale.tar.gz" -C "$vale_dir" vale
test -x "$vale_dir/vale"
export PATH="$vale_dir:$PATH"
"$vale_dir/vale" --version
cargo test --locked --test vale -- --ignored
