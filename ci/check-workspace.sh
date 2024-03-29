#!/bin/bash
set -e

# Check for required arguments.
if [ $# -lt 2 ]; then
  echo "Usage: ci/check-worksapce.sh [OPTIONS] NAME MANIFEST-PATH" >&2
  echo "" >&2
  echo "OPTIONS:" >&2
  echo "    --full             Run clippy, fmt and doc checks on top of tests" >&2
  echo "    --no-all-features  Disable the --all-features flag to cargo commands" >&2
  exit 1
fi

# Parse CLI arguments.
ALL_FEATURES="--all-features"
FULL_MODE=no

while [ $# -gt 0 ]; do
  arg=$1
  shift
  case "${arg}" in
    --full) FULL_MODE=yes;;
    --no-all-features) ALL_FEATURES="";;
    *)
      NAME="${arg}"
      MANIFEST="$1"
      shift
      ;;
  esac
done

# GitHub Actions log group support, when running in CI only.
log_group() {
  if [ -n "${CI}" ]; then
    echo "::group::$1"
  else
    echo "$1"
  fi
}
log_group_end() {
  if [ -n "${CI}" ]; then
    echo "::endgroup::"
  fi
}

# Build, test, clippy, format stages.
log_group "Build ${NAME} packages"
cargo build "${ALL_FEATURES}" --manifest-path "${MANIFEST}"
log_group_end

log_group "Run ${NAME} tests"
cargo test "${ALL_FEATURES}" --manifest-path "${MANIFEST}"
log_group_end

# Stop early if not in full mode.
if [ "${FULL_MODE}" != "yes" ]; then
  exit 0
fi

log_group "Run ${NAME} clippy"
cargo clippy "${ALL_FEATURES}" --manifest-path "${MANIFEST}" -- -D warnings
log_group_end

log_group "Check ${NAME} formatting"
# Cargo fmt behaves oddly with "Cargo.toml" for the manifest path.
# Pass the manifest path only if not the root crate.
# Issue: https://github.com/rust-lang/rustfmt/issues/4432
if [ "${MANIFEST}" == "Cargo.toml" ]; then
  cargo fmt --verbose -- --check
else
  cargo fmt --manifest-path "${MANIFEST}" --verbose -- --check
fi
log_group_end

log_group "Run ${NAME} doc"
# Use --no-deps to skip documenting 3rd-party crates but also --all for all worksapce members.
cargo doc --manifest-path "${MANIFEST}" "${ALL_FEATURES}" --no-deps --all
log_group_end
