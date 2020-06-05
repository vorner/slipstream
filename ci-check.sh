#!/bin/sh

# We try to support some older versions of rustc. However, the support is
# tiered a bit. Our dev-dependencies do *not* guarantee that old minimal
# version. So we don't do tests on the older ones. Also, the
# signal-hook-registry supports older rustc than we signal-hook.

set -ex

export PATH="$PATH":~/.cargo/bin
export RUST_BACKTRACE=1
export CARGO_INCREMENTAL=1

cargo build --all
cargo clippy --all --tests -- --deny clippy::all
cargo fmt

# Sometimes nightly doesn't have clippy or rustfmt, so don't try that there.
if [ "$TRAVIS_RUST_VERSION" = nightly ] ; then
	exit
fi

cargo test --all
cargo doc --no-deps
