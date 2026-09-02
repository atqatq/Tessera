# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0
#
# One entry point for every gate. `make check` is what CI runs and what a
# contributor runs before pushing; there is no second, divergent list.

.DEFAULT_GOAL := check
.PHONY: check fmt fmt-fix clippy test-rust test-py vectors manifests reuse clean

# --- gates (fast first, so failures surface early) -----------------------

fmt:            ## cargo fmt --check (rust code shape)
	cd rust && cargo fmt --all -- --check

clippy:         ## clippy with the workspace robustness lints, deny warnings
	cd rust && cargo clippy --workspace --all-targets -- -D warnings

test-rust:      ## rust test suite, conformance vectors included
	cd rust && cargo test --workspace

test-py:        ## python reference suite (168 tests)
	pytest reference/python -q

manifests:      ## spoke manifests: independence rule + schema checks
	python tools/check_manifests.py 'spokes/*/manifest.json'

reuse:          ## REUSE licensing: every file carries its SPDX data
	reuse lint

# vectors are exercised by both test suites (test_conformance.py and
# crates/scor-expr/tests/conformance.rs read the same file), so no
# separate target: a drift breaks the build, not a document.

check: fmt clippy test-rust test-py manifests reuse
	@echo "all gates green"

# --- conveniences ---------------------------------------------------------

fmt-fix:        ## apply rustfmt (the only auto-formatter)
	cd rust && cargo fmt --all

clean:
	cd rust && cargo clean
	rm -rf reference/python/.pytest_cache reference/python/**/__pycache__
