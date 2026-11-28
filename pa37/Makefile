# RustGM Programming Assignment 37: Reproducible self-hosting (inception)
# Modeled on CPPGM PA35: stage0 (seed) -> stage1 (self) -> PA1..PA24 ladder -> stage2 (inception) -> byte compare.
ROOT := $(abspath ..)
PYTHON ?= python3
PA37_WORK ?= $(ROOT)/obj/pa37
PA37_OPT_LEVEL ?= 1
PA37_LADDER_THROUGH ?= 24
RUSTGM_TEST_JOBS ?= 4
PA37_ARGS ?=

ifdef RUSTGM_RUST_SRC
PA37_RUST_SRC_ARG := --rust-src "$(RUSTGM_RUST_SRC)"
else
PA37_RUST_SRC_ARG :=
endif

ifdef RUSTGM_STAGE0
PA37_STAGE0_ARG := --stage0 "$(RUSTGM_STAGE0)"
else
PA37_STAGE0_ARG :=
endif

ifdef RUSTGM_LINKER
PA37_LINKER_ARG := --linker "$(RUSTGM_LINKER)"
else
PA37_LINKER_ARG :=
endif

DRIVER = $(PYTHON) "$(ROOT)/scripts/pa37_self_host.py" \
	--work "$(PA37_WORK)" \
	--optimization "$(PA37_OPT_LEVEL)" \
	--jobs "$(RUSTGM_TEST_JOBS)" \
	$(PA37_STAGE0_ARG) $(PA37_LINKER_ARG) $(PA37_RUST_SRC_ARG) $(PA37_ARGS)

PA_STAGES := $(shell seq 1 30)
TEST_PA_TARGETS := $(addprefix test-pa,$(PA_STAGES))
TEST_THROUGH_TARGETS := $(addprefix test-through-pa,$(PA_STAGES))

.PHONY: all stage0 stage1 rustgm-self stage2 rustgm-inception compare compare-rustgm-inception \
	negative-controls inception test check clean $(TEST_PA_TARGETS) $(TEST_THROUGH_TARGETS)
.DEFAULT_GOAL := test

all: stage1

stage0:
	$(DRIVER) stage0

stage1 rustgm-self:
	$(DRIVER) stage1

stage2 rustgm-inception:
	$(DRIVER) stage2

compare compare-rustgm-inception:
	$(DRIVER) compare

negative-controls:
	$(DRIVER) negative-controls

$(TEST_PA_TARGETS): test-pa%:
	$(DRIVER) ladder --stage pa$*

$(TEST_THROUGH_TARGETS): test-through-pa%:
	$(DRIVER) ladder --through $*

test inception check:
	$(DRIVER) test --through "$(PA37_LADDER_THROUGH)" --build-stage0

clean:
	rm -rf "$(PA37_WORK)"
