#!/bin/sh

set -ex

export PATH="$PATH":~/.cargo/bin
export RUST_BACKTRACE=1
export CARGO_INCREMENTAL=1

cargo build
cargo test

# Sometimes nightly doesn't have clippy or rustfmt, so don't try that there.
if [ "$TRAVIS_RUST_VERSION" = nightly ] ; then
	cargo test --all --benches
	exit
fi

cargo doc --no-deps
cargo clippy --tests -- --deny clippy::all
cargo fmt
