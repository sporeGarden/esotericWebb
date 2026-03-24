# SPDX-License-Identifier: AGPL-3.0-or-later
# Esoteric Webb — reproducible quality gates

.PHONY: all check fmt clippy test doc deny clean

export CARGO_TARGET_DIR ?= $(HOME)/.cargo-build/esotericWebb/target
export CARGO_HOME ?= $(HOME)/.cargo

all: check deny

check: fmt clippy test doc

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --lib --tests

doc:
	cargo doc --workspace --no-deps

deny:
	cargo deny check

clean:
	cargo clean
