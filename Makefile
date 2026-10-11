PYTHON ?= python3
CARGO ?= cargo
ROOT := $(CURDIR)
export CARGO_TARGET_DIR := $(ROOT)/obj/rust
ACTIVE_PAS := pa1 pa2 pa3
RUSTGM_TEST_APP ?= $(ROOT)/dev/rustgm
REF_TEST_APP ?= $(ROOT)/dev/rustgm-ref

.PHONY: all test test-report ref-test lint setup-audit audit-rust test-harness test-backend test-upstream test-rustc perf perf-literals clean
all:
	$(CARGO) build --manifest-path dev/Cargo.toml --release --locked
test: all
	$(CARGO) test --manifest-path dev/Cargo.toml --locked
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(RUSTGM_TEST_APP)" || status=1; done; exit $$status
test-report: all
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(RUSTGM_TEST_APP)" || status=1; done; exit $$status
	@echo 'PA4-PA38: planned, not included in acceptance totals'
test-pa%: all
	$(PYTHON) scripts/course.py pa$* --app "$(RUSTGM_TEST_APP)"
ref-test: all
	@status=0; for stage in $(ACTIVE_PAS); do $(PYTHON) scripts/course.py $$stage --app "$(REF_TEST_APP)" --bless || status=1; done; exit $$status
ref-test-pa%: all
	$(PYTHON) scripts/course.py pa$* --app "$(REF_TEST_APP)" --bless
lint:
	$(CARGO) fmt --manifest-path dev/Cargo.toml --all -- --check
	$(CARGO) clippy --manifest-path dev/Cargo.toml --all-targets --all-features --locked -- -D warnings
setup-audit:
	$(PYTHON) -m venv obj/audit-venv
	obj/audit-venv/bin/pip install -r scripts/requirements-audit.txt
audit-rust:
	obj/audit-venv/bin/python scripts/rust_audit.py --root .
test-harness:
	$(PYTHON) -m unittest discover -s scripts/tests -v
	$(PYTHON) scripts/check_layout.py
test-backend:
	$(PYTHON) scripts/backend.py --test
test-upstream:
	$(PYTHON) scripts/upstream.py --verify
test-rustc: all
	$(PYTHON) scripts/check_rustc.py
	$(PYTHON) scripts/check_literals.py --app "$(RUSTGM_TEST_APP)"
	$(PYTHON) scripts/check_syntax.py pa3 --app "$(RUSTGM_TEST_APP)"
perf: all
	$(PYTHON) scripts/measure_lexer.py
perf-literals: all
	$(PYTHON) scripts/measure_lexer.py --stage literals
clean:
	$(CARGO) clean --manifest-path dev/Cargo.toml
