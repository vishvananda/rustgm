ROOT := $(abspath ..)
PA := $(notdir $(CURDIR))
PYTHON ?= python3
RUSTGM_TEST_JOBS ?= 2
CONSUMER_ARGS ?=

ifeq ($(origin RUSTGM_TEST_APP),undefined)
RUSTGM_TEST_APP := $(ROOT)/obj/student/release/rustgm
test: all
endif

.PHONY: all test check
.DEFAULT_GOAL := test
all:
	$(MAKE) -C "$(ROOT)" all

test:
	$(PYTHON) "$(ROOT)/scripts/grade_consumers.py" $(PA) \
		--app "$(abspath $(RUSTGM_TEST_APP))" \
		--jobs "$(RUSTGM_TEST_JOBS)" $(CONSUMER_ARGS)

check: test
