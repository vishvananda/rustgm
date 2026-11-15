/* Original RustGM maintainer input. It follows the public C interface of the
 * pinned CPPGM LowIR library (lowir_interface.h); no upstream source is copied.
 *
 * Builds one module in process: a read-only `rg_library_seed` holding 40 and
 * `rg_library_answer(x)`, which returns the seed plus `x`. It compiles the
 * module at O0 and writes the object bytes to argv[1]; library-observer.c
 * links that object and calls it. No text, subprocess or temporary file is
 * involved on the library side. */
#include "lowir_interface.h"

#include <stdio.h>

_Static_assert(sizeof(LowirValue) == 8, "value record");
_Static_assert(sizeof(LowirInstruction) == 64, "instruction record");

static LowirValue value(uint32_t kind, uint32_t index)
{
	LowirValue result = { kind, 0, 0, index };
	return result;
}

/* An i64 instruction of the only block (location 0). */
static LowirInstruction instruction(uint8_t op, uint32_t result)
{
	LowirInstruction inst = { 0 };
	inst.op = op;
	inst.result = result;
	inst.type = CPPGM_TYPE_I64;
	return inst;
}

static int build(CppgmLowir* module)
{
	uint32_t seed = cppgm_lowir_symbol(module, "rg_library_seed", 15);
	uint32_t answer = cppgm_lowir_symbol(module, "rg_library_answer", 17);
	CppgmType parameter = CPPGM_TYPE_I64;
	uint32_t signature = cppgm_lowir_signature(module, CPPGM_TYPE_I64, &parameter, 1, 0);
	if (seed == CPPGM_NO_ID || answer == CPPGM_NO_ID || signature == CPPGM_NO_ID)
		return 0;
	CppgmDataItem data = { 0 };
	data.type = CPPGM_TYPE_I64;
	data.bits.low = 40;
	/* Strong binding (2); the global is read-only storage (1). */
	if (!cppgm_lowir_global(module, seed, 0, 2, 1, &data, 1, 0) ||
		!cppgm_lowir_function(module, answer, signature, 2, 1))
		return 0;
	LowirBodyCounts counts = { 3, 0, 0, 0, 1 };
	LowirBodyBuffers buffers;
	if (!cppgm_lowir_body_reserve(module, answer, &counts, &buffers))
		return 0;
	/* The parameter is temporary 0; the two definitions follow it. */
	buffers.instructions[0] = instruction(CPPGM_OP_LOAD, 1);
	buffers.instructions[0].ops[0] = value(CPPGM_VALUE_SYMBOL, seed);
	buffers.instructions[1] = instruction(CPPGM_OP_BINARY, 2);
	buffers.instructions[1].sub = CPPGM_BINARY_ADD;
	buffers.instructions[1].ops[0] = value(CPPGM_VALUE_TEMP, 1);
	buffers.instructions[1].ops[1] = value(CPPGM_VALUE_TEMP, 0);
	buffers.instructions[2] = instruction(CPPGM_OP_RETURN, CPPGM_NO_ID);
	buffers.instructions[2].ops[0] = value(CPPGM_VALUE_TEMP, 2);
	buffers.layout[0] = 0;
	return cppgm_lowir_body_commit(module, answer, &counts, 3) && cppgm_lowir_validate(module);
}

int main(int argc, char** argv)
{
	if (argc != 2)
		return 2;
	/* A wrong interface version or target must be refused, not guessed. */
	if (cppgm_lowir_create(CPPGM_LOWIR_ABI_VERSION + 1, 0) != NULL ||
		cppgm_lowir_create(CPPGM_LOWIR_ABI_VERSION, 99) != NULL)
		return 3;
	CppgmLowir* module = cppgm_lowir_create(CPPGM_LOWIR_ABI_VERSION, 0);
	if (module == NULL)
		return 3;
	/* Position-independent code, as the host C driver links by default. */
	if (!build(module) || !cppgm_lowir_compile(module, 0, 1))
	{
		fprintf(stderr, "%s\n", cppgm_lowir_error(module));
		cppgm_lowir_destroy(module);
		return 4;
	}
	const uint8_t* bytes = cppgm_lowir_object_data(module);
	uint64_t size = cppgm_lowir_object_size(module);
	FILE* file = fopen(argv[1], "wb");
	int written = file != NULL && size != 0 && fwrite(bytes, 1, size, file) == size;
	if (file != NULL && fclose(file))
		written = 0;
	cppgm_lowir_destroy(module);
	return written ? 0 : 5;
}
