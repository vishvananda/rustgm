ROOT := $(abspath ..)
PA := $(notdir $(CURDIR))
PYTHON ?= python3
RUSTGM_TEST_APP ?= $(ROOT)/obj/rust/release/rustgm
RUSTGM_TEST_JOBS ?= 2
CONSUMER_ARGS ?=

.PHONY: all test check
all:
	$(MAKE) -C $(ROOT) all

test: all
	$(PYTHON) $(ROOT)/scripts/storage.py run --prune-under obj/tests/$(PA) -- \
		$(PYTHON) $(ROOT)/scripts/grade_consumers.py $(PA) \
		--app "$(abspath $(RUSTGM_TEST_APP))" \
		--jobs "$(RUSTGM_TEST_JOBS)" $(CONSUMER_ARGS)

check: test
