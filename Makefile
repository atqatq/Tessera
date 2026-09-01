# Tessera — developer entry points (A8).
#
#   make setup    one command from a clean machine to a working env
#   make test     the full local suite (fast: well under two minutes)
#   make lint     fmt --check + clippy -D warnings
#   make check    everything CI runs, locally — the same bar, no excuses
#
# CI runs exactly what these targets run; if a gate only exists in CI,
# it is not a gate, it is a surprise.

SHELL := /bin/bash
RUST_PIN := 1.98.0

.PHONY: help setup toolchain test python-test lint fmt clippy coverage mutation docs demo vectors clean check

help:
	@echo "make setup    - install toolchain + editable reference package"
	@echo "make test     - cargo test + python unittest"
	@echo "make lint     - fmt --check + clippy -D warnings"
	@echo "make check    - lint + test + vector freshness (what CI runs)"
	@echo "make coverage - kernel-crate coverage with an 85% floor"
	@echo "make mutation - cargo-mutants on access + ledger"
	@echo "make docs     - build the docs site (mdbook)"
	@echo "make demo     - regenerate the terminal recording (needs vhs)"

setup: toolchain
	python3 -m pip install -e reference/python
	@echo "setup complete: cargo $$(cargo --version 2>/dev/null || echo MISSING)"

toolchain:
	@command -v cargo >/dev/null 2>&1 || { \
		echo "installing Rust $(RUST_PIN) via rustup"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
			--profile minimal --default-toolchain $(RUST_PIN) \
			--component clippy,rustfmt; \
	}

test:
	cargo test --workspace
	python3 -m unittest discover -s reference/python/tests

python-test:
	python3 -m unittest discover -s reference/python/tests

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

lint: fmt clippy

vectors:
	python3 reference/python/tools/gen_vectors.py
	git diff --exit-code reference/python/vectors

coverage:
	cargo llvm-cov --workspace --fail-under-lines 85

mutation:
	cargo mutants --package tessera-access --package tessera-ledger --no-shuffle -j 2

docs:
	mdbook build docs/site

# Regenerates the terminal recording from its committed source (C2).
demo:
	vhs docs/assets/demo.tape

# `make check` is the local mirror of CI (E7): every gate a PR faces,
# runnable in one command on a clean machine.
check: lint test vectors
	@echo "check complete — same bar as CI"

clean:
	cargo clean
	rm -rf docs/site/book
