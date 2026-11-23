PYTHON ?= python3
CARGO ?= cargo
CXX ?= c++
export CARGO_TARGET_DIR := $(CURDIR)/obj/student

.PHONY: all setup-backend backend-example test-upstream
all:
	$(CARGO) build --release --locked

setup-backend:
	$(PYTHON) scripts/setup_backend.py

backend-example:
	@set -eu; \
	config=$$($(PYTHON) scripts/backend.py --library-config); \
	folder=$$(printf '%s\n' "$$config" | sed -n 's/^folder=//p'); \
	mkdir -p obj/lowir-example; \
	$(CC) $(CPPFLAGS) $(CFLAGS) -std=c11 -Wall -Wextra -Werror -I"$$folder" \
		-c examples/lowir-library/return42.c -o obj/lowir-example/producer.o; \
	$(CXX) $(LDFLAGS) obj/lowir-example/producer.o "$$folder/libcppgm_lowir.a" \
		-pthread -o obj/lowir-example/producer; \
	obj/lowir-example/producer obj/lowir-example/return42.o; \
	$(CC) $(LDFLAGS) obj/lowir-example/return42.o -o obj/lowir-example/return42; \
	status=0; obj/lowir-example/return42 || status=$$?; \
	test "$$status" -eq 42; \
	printf 'LowIR example returned %s\n' "$$status"

test-upstream:
	$(PYTHON) scripts/upstream.py --verify
