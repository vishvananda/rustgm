include ../scripts/assignment.mk

test: test-driver
.PHONY: test-driver
test-driver: all
	$(PYTHON) $(ROOT)/scripts/grade_pa34_driver.py --app "$(RUSTGM_TEST_APP)"
