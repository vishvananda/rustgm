ROOT := $(abspath ..)
PA := $(notdir $(CURDIR))
PYTHON ?= python3
RUSTGM_TEST_APP ?= $(ROOT)/dev/rustgm
REF_TEST_APP ?= $(ROOT)/dev/rustgm-ref
.PHONY: all test ref-test check
all:
	$(MAKE) -C $(ROOT) all
test: all
	$(PYTHON) $(ROOT)/scripts/course.py $(PA) --app "$(RUSTGM_TEST_APP)"
ref-test: all
	$(PYTHON) $(ROOT)/scripts/course.py $(PA) --app "$(REF_TEST_APP)" --bless
check: test
