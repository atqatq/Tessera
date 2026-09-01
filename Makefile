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

# rustup-installed-but-not-on-PATH is the normal state right after
# rustup-init: it edits future shells, never the current one. Make cargo
# visible to every recipe in this invocation; a no-op when cargo is
# already found on PATH.
ifeq ($(shell command -v cargo 2>/dev/null),)
ifneq ($(wildcard $(HOME)/.cargo/bin/cargo),)
export PATH := $(HOME)/.cargo/bin:$(PATH)
endif
endif

# Python runner: `make setup` creates .venv only when the system python
# is externally managed (PEP 668 — Debian 12+, Ubuntu 23.04+ stock
# python). Once the venv exists it wins; otherwise the system python is
# used exactly as before. Evaluated per invocation, so any make target
# after `make setup` picks the right interpreter up automatically.
PY := $(shell [ -x .venv/bin/python ] && echo .venv/bin/python || echo python3)

.PHONY: help setup toolchain venv test python-test lint fmt clippy coverage mutation docs demo vectors clean check

help:
	@echo "make setup    - install toolchain + editable Python reference"
	@echo "make test     - cargo test + reference unittests"
	@echo "make lint     - fmt --check + clippy -D warnings"
	@echo "make check    - lint + test + vector freshness (what CI runs)"
	@echo "make coverage - kernel-crate coverage with an 85% floor"
	@echo "make mutation - cargo-mutants on access + ledger"
	@echo "make docs     - build the docs site (mdbook)"
	@echo "make demo     - regenerate the terminal recording (needs vhs)"

setup: toolchain venv
	@if [ -x .venv/bin/pip ]; then PIP=.venv/bin/pip; else PIP="python3 -m pip"; fi; \
		$$PIP install -e reference/python
	@if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi; \
		printf 'setup complete: %s + %s\n' "$$(cargo --version)" "$$($$PY --version)"

toolchain:
	@command -v cargo >/dev/null 2>&1 || { \
		echo "installing Rust $(RUST_PIN) via rustup"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
			--profile minimal --default-toolchain $(RUST_PIN) \
			--component clippy,rustfmt; \
	}
	@PATH="$(HOME)/.cargo/bin:$${PATH}" command -v cargo >/dev/null 2>&1 || { \
		echo "toolchain: rustup ran but cargo is still not available" >&2; \
		echo "  open a new shell (so ~/.cargo/env is sourced) and re-run 'make setup'" >&2; \
		exit 1; \
	}
	@printf 'toolchain: %s\n' "$$(cargo --version)"

# PEP 668: Debian and Ubuntu mark the system python externally managed,
# so a global `pip install -e` refuses to run there. Detect the marker
# file and build an isolated venv only when needed; CI images and
# pyenv/conda machines keep using the system python directly.
venv:
	@command -v python3 >/dev/null 2>&1 || { \
		echo "python3 not found - install Python 3.11+ first" >&2; exit 1; \
	}
	@if [ ! -x .venv/bin/python ] && python3 -c 'import os, sysconfig; raise SystemExit(0 if os.path.exists(os.path.join(sysconfig.get_path("stdlib"), "EXTERNALLY-MANAGED")) else 1)' 2>/dev/null; then \
		echo "externally-managed python (PEP 668) detected - creating .venv"; \
		if ! python3 -m venv .venv; then \
			rm -rf .venv; \
			echo "ensurepip unavailable (python3-venv missing) - bootstrapping pip via get-pip.py"; \
			python3 -m venv --without-pip .venv || { echo "venv creation failed" >&2; exit 1; }; \
			get_pip="$${TMPDIR:-/tmp}/get-pip-$$$.py"; \
			curl -sSf https://bootstrap.pypa.io/get-pip.py -o "$$get_pip" && .venv/bin/python "$$get_pip" -q; rc=$$?; rm -f "$$get_pip"; \
			[ $$rc -eq 0 ] || { echo "pip bootstrap failed - install python3-venv and re-run 'make setup'" >&2; rm -rf .venv; exit 1; }; \
		fi; \
	fi

test:
	cargo test --workspace
	$(PY) -m unittest discover -s reference/python/tests

python-test:
	$(PY) -m unittest discover -s reference/python/tests

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

lint: fmt clippy

vectors:
	$(PY) reference/python/tools/gen_vectors.py
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
