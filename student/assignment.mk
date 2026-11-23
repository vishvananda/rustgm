ROOT := $(abspath ..)
PA := $(notdir $(CURDIR))
PYTHON ?= python3

ifeq ($(origin RUSTGM_TEST_APP),undefined)
RUSTGM_TEST_APP := $(ROOT)/obj/student/release/rustgm
test: all
endif

.PHONY: all test check
.DEFAULT_GOAL := test
all:
	$(MAKE) -C "$(ROOT)" all

test:
	$(PYTHON) "$(ROOT)/scripts/course.py" $(PA) --app "$(abspath $(RUSTGM_TEST_APP))"

check: test
