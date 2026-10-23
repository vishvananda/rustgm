PYTHON ?= python3
CARGO ?= cargo
GUARDED = $(PYTHON) scripts/storage.py run --
export CARGO_INCREMENTAL := 0
ROOT := $(CURDIR)
export CARGO_TARGET_DIR := $(ROOT)/obj/rust
ACTIVE_PAS := pa1 pa2 pa3 pa4 pa5 pa6 pa7 pa8 pa9 pa10 pa11 pa12 pa13 pa14 pa15 pa16 pa17 pa18 pa19 pa20 pa21 pa22 pa23 pa24
RUSTGM_TEST_APP ?= $(ROOT)/dev/rustgm
REF_TEST_APP ?= $(ROOT)/dev/rustgm-ref

.PHONY: all test test-debug test-debug-invariants test-maintainer test-report ref-test lint setup-audit audit-rust test-harness test-backend test-native test-maintainer-native test-layouts test-upstream test-rustc perf perf-literals perf-ast perf-crate-ast perf-names perf-expanded perf-expanded-crate perf-types perf-layouts perf-body-ir perf-ownership perf-lowir clean tidy tidy-build check-space
all:
	$(PYTHON) scripts/storage.py run --no-prune -- $(CARGO) build --manifest-path dev/Cargo.toml --release --locked
test:
	$(GUARDED) $(CARGO) test --manifest-path dev/Cargo.toml --release --locked
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(RUSTGM_TEST_APP)" || status=1; done; exit $$status
test-debug:
	$(GUARDED) $(CARGO) test --manifest-path dev/Cargo.toml --locked
test-debug-invariants:
	$(GUARDED) $(CARGO) test --manifest-path dev/Cargo.toml --locked --lib
test-maintainer: test-backend
	$(GUARDED) $(CARGO) test --manifest-path dev/Cargo.toml --release --locked -- --include-ignored
test-report: all
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(RUSTGM_TEST_APP)" || status=1; done; exit $$status
	@echo 'PA25-PA44: planned, not included in acceptance totals'
test-pa%: all
	$(PYTHON) scripts/course.py pa$* --app "$(RUSTGM_TEST_APP)"
ref-test: all
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(REF_TEST_APP)" --bless || status=1; done; exit $$status
ref-test-pa%: all
	$(PYTHON) scripts/course.py pa$* --app "$(REF_TEST_APP)" --bless
lint:
	$(CARGO) fmt --manifest-path dev/Cargo.toml --all -- --check
	$(PYTHON) scripts/storage.py run --no-prune -- $(CARGO) clippy --manifest-path dev/Cargo.toml --all-targets --all-features --locked -- -D warnings
setup-audit:
	$(PYTHON) -m venv obj/audit-venv
	obj/audit-venv/bin/pip install -r scripts/requirements-audit.txt
audit-rust:
	obj/audit-venv/bin/python -m unittest discover -s scripts/audit_tests -v
	obj/audit-venv/bin/python scripts/rust_audit.py --root .
test-harness:
	$(PYTHON) -m unittest discover -s scripts/tests -v
	$(PYTHON) scripts/maintainer_suites.py check
	$(PYTHON) scripts/check_layout.py
.PHONY: test-host-builds
test-host-builds:
	$(PYTHON) scripts/storage.py run --prune-under obj/qualification/host-graph -- $(PYTHON) scripts/check_host_graph.py
test-backend:
	$(PYTHON) scripts/backend.py --test
test-maintainer-native: test-backend
	$(GUARDED) $(CARGO) test --manifest-path dev/Cargo.toml --release --locked -- --ignored
test-native: test-maintainer-native
	$(GUARDED) $(PYTHON) scripts/check_native.py pa11 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa12 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa13 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa14 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa16 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa17 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa18 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa19 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa20 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa21 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa22 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa23 --app "$(RUSTGM_TEST_APP)"
	$(GUARDED) $(PYTHON) scripts/check_native.py pa24 --app "$(RUSTGM_TEST_APP)"
test-native-pa%: all
	$(GUARDED) $(PYTHON) scripts/check_native.py pa$* --app "$(RUSTGM_TEST_APP)"
test-layouts: all
	$(PYTHON) scripts/check_layouts.py pa12 --app "$(RUSTGM_TEST_APP)"
test-upstream:
	$(PYTHON) scripts/upstream.py --verify
test-rustc: all test-layouts
	$(PYTHON) scripts/check_rustc.py
	$(PYTHON) scripts/check_literals.py --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa3 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa4 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa5 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa6 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa7 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa8 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa9 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa10 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa11 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa12 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa13 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa14 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa15 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa16 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa17 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa18 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa19 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa20 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa21 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa22 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa23 --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa24 --app "$(RUSTGM_TEST_APP)"
perf: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py
perf-literals: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage literals
perf-ast: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage ast
perf-crate-ast: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage crate-ast
perf-names: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage names
perf-expanded: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage expanded
perf-expanded-crate: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage expanded-crate
perf-types: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage types
perf-layouts: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage layouts
perf-body-ir: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage body-ir
perf-lowir: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage lowir
perf-ownership: all
	$(GUARDED) $(PYTHON) scripts/measure_lexer.py --stage ownership
clean:
	$(CARGO) clean --manifest-path dev/Cargo.toml

check-space:
	$(PYTHON) scripts/storage.py check
tidy:
	$(PYTHON) scripts/storage.py prune --apply
tidy-build:
	$(PYTHON) scripts/storage.py prune --apply --build-cache
