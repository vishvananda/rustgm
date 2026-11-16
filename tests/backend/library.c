/* Original RustGM maintainer input. It follows the public C interface of the
 * pinned CPPGM LowIR library (lowir_interface.h, ABI version 4); no upstream
 * source is copied.
 *
 * Usage: producer OBJECT TEXT RELEASE fixed|pic
 *
 * Builds one module in process, writes the library's text view of it to TEXT,
 * compiles it at O0 as fixed or position-independent code and writes the
 * object bytes to OBJECT. No LowIR text is read and no subprocess or temporary
 * file is involved on the library side. library-observer.c links the object
 * and calls every function:
 *
 *   rg_library_seed          a read-only i64 holding 40
 *   rg_library_answer(x)     the seed plus x
 *   rg_library_swap(p)       takes {i64, f64}, returns {f64, i64}:
 *                            `classes=` and `result_classes=`, with the
 *                            default linkage that RustGM's text states
 *   rg_library_apply(f, x, p)
 *                            returns f(p), called through a pointer: a call
 *                            boundary `as (...)` that states both classes,
 *                            and an object temporary for its result
 *   rg_library_tally(n, d)   calls the variadic rg_host_tally(1, {n, d}):
 *                            `arity=variadic` and a call-site `classes=1:`
 *   rg_library_stream(d, s)  one `movntps` through an assembly template
 *   rg_library_turn(r)       takes {{f32, f32}, i64}, returns {i64, f32[2]}:
 *                            record types, whose `classes=` and
 *                            `result_classes=` the library derives
 *   rg_library_stop(code)    `return=noreturn`: calls exit(code)
 *   rg_library_match(a, b, x)
 *                            the sign bits of the bytes equal in two v128
 *                            values and in x's dword broadcast: `vector`
 *                            pcmpeqb, fromi32, pshufd 0, pand and pmovmskb
 *   rg_library_tick()        counts in the thread-local rg_library_ticks,
 *                            which starts at 10 in each thread:
 *                            `storage=thread_local`
 */
#include "lowir_interface.h"

#include <stdio.h>
#include <string.h>

/* The one interface version this producer is written for. */
#define RG_ABI_VERSION 4u
_Static_assert(CPPGM_LOWIR_ABI_VERSION == RG_ABI_VERSION, "interface version");
_Static_assert(sizeof(LowirValue) == 8, "value record");
_Static_assert(sizeof(LowirInstruction) == 64, "instruction record");
_Static_assert(sizeof(CppgmParameter) == 24, "parameter record");
_Static_assert(sizeof(CppgmBoundary) == 12, "boundary record");
_Static_assert(sizeof(CppgmArgumentFact) == 16, "argument fact record");
_Static_assert(sizeof(CppgmField) == 24, "record field");

#define NAME(text) text, (uint32_t)(sizeof text - 1)
/* obj<16x8>: the kind, the alignment's log2 and the size. */
#define PAIR ((CppgmType)CPPGM_TYPE_OBJ | ((CppgmType)3 << 8) | ((CppgmType)16 << 16))
#define INTEGER_SSE CPPGM_CLASSES(CPPGM_EIGHTBYTE_INTEGER, CPPGM_EIGHTBYTE_SSE)
#define SSE_INTEGER CPPGM_CLASSES(CPPGM_EIGHTBYTE_SSE, CPPGM_EIGHTBYTE_INTEGER)

static CppgmLowir* module;

static LowirValue value(uint32_t kind, uint32_t index)
{
	LowirValue result = { kind, 0, 0, index };
	return result;
}

/* An instruction of the only block (location 0). */
static LowirInstruction instruction(uint8_t op, uint32_t result, CppgmType type)
{
	LowirInstruction inst = { 0 };
	inst.op = op;
	inst.result = result;
	inst.type = type;
	return inst;
}

static LowirInstruction unary(uint8_t op, uint32_t result, CppgmType type, LowirValue operand)
{
	LowirInstruction inst = instruction(op, result, type);
	inst.ops[0] = operand;
	return inst;
}

static LowirInstruction binary(uint8_t op, uint32_t result, CppgmType type, LowirValue left, LowirValue right)
{
	LowirInstruction inst = unary(op, result, type, left);
	inst.ops[1] = right;
	return inst;
}

/* `copyobj 16x8 source, destination`. */
static LowirInstruction copy_pair(LowirValue source, LowirValue destination)
{
	LowirInstruction inst = binary(CPPGM_OP_COPY_OBJECT, CPPGM_NO_ID, CPPGM_TYPE_VOID, source, destination);
	inst.immediate = 16;
	inst.sub = 3;
	return inst;
}

/* A call whose arguments are `count` operands of the pool from `first`. */
static LowirInstruction call(uint32_t result, CppgmType type, LowirValue callee, uint32_t first, uint32_t count)
{
	LowirInstruction inst = unary(CPPGM_OP_CALL, result, type, callee);
	inst.extra_first = first;
	inst.extra_count = count;
	return inst;
}

static CppgmParameter parameter(CppgmType type, uint8_t classes)
{
	CppgmParameter result = { 0 };
	result.type = type;
	result.classes = classes;
	return result;
}

/* A strong definition, or an undefined external, of a new symbol. */
static uint32_t function(const char* name, uint32_t length, uint32_t signature, uint32_t form)
{
	uint32_t symbol = cppgm_lowir_symbol(module, name, length);
	uint32_t binding = form == CPPGM_FUNCTION_DEFINITION ? CPPGM_BINDING_STRONG : CPPGM_BINDING_DEFAULT;
	if (symbol == CPPGM_NO_ID || signature == CPPGM_NO_ID ||
		!cppgm_lowir_function(module, symbol, signature, binding, form))
		return CPPGM_NO_ID;
	return symbol;
}

/* Buffers for a body of one block. */
static int reserve(uint32_t symbol, const LowirBodyCounts* counts, LowirBodyBuffers* buffers)
{
	if (symbol == CPPGM_NO_ID || !cppgm_lowir_body_reserve(module, symbol, counts, buffers))
		return 0;
	buffers->layout[0] = 0;
	return 1;
}

static LowirSlot pair_slot(void)
{
	LowirSlot slot = { 0 };
	slot.type = PAIR;
	return slot;
}

/* The ABI version 2 subset: a global and a function that loads it. */
static int answer(void)
{
	uint32_t seed = cppgm_lowir_symbol(module, NAME("rg_library_seed"));
	CppgmType argument = CPPGM_TYPE_I64;
	uint32_t symbol = function(NAME("rg_library_answer"),
		cppgm_lowir_signature(module, CPPGM_TYPE_I64, &argument, 1, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	CppgmDataItem data = { 0 };
	data.type = CPPGM_TYPE_I64;
	data.bits.low = 40;
	data.kind = CPPGM_DATA_SCALAR;
	LowirBodyCounts counts = { 3, 0, 0, 0, 1 };
	LowirBodyBuffers buffers;
	if (seed == CPPGM_NO_ID ||
		!cppgm_lowir_global(module, seed, CPPGM_TYPE_VOID, CPPGM_BINDING_STRONG, CPPGM_STORAGE_READONLY, &data, 1,
			CPPGM_GLOBAL_DEFINITION) ||
		!reserve(symbol, &counts, &buffers))
		return 0;
	/* The parameter is temporary 0; the two definitions follow it. */
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_LOAD, 1, CPPGM_TYPE_I64, value(CPPGM_VALUE_SYMBOL, seed));
	code[1] = binary(CPPGM_OP_BINARY, 2, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 1), value(CPPGM_VALUE_TEMP, 0));
	code[1].sub = CPPGM_BINARY_ADD;
	code[2] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 2));
	return cppgm_lowir_body_commit(module, symbol, &counts, 3);
}

/* function @rg_library_swap(%p : obj<16x8> [classes=integer_sse])
 *     -> obj<16x8> [result_classes=sse_integer]
 * The integer arrives in a general register and the double in an SSE one;
 * they return in the opposite order. Without the classes both eightbytes
 * would be INTEGER and the caller would read the wrong registers. */
static int swap(void)
{
	CppgmParameter incoming = parameter(PAIR, INTEGER_SSE);
	CppgmBoundary facts = { 0 };
	facts.result_classes = SSE_INTEGER;
	uint32_t symbol = function(NAME("rg_library_swap"), cppgm_lowir_boundary(module, PAIR, &incoming, 1, &facts),
		CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 10, 0, 1, 2, 1 };
	LowirBodyBuffers buffers;
	/* A library function has C linkage unless a symbol fact states another. */
	if (symbol == CPPGM_NO_ID ||
		!cppgm_lowir_symbol_fact(module, symbol, CPPGM_KEY_LINKAGE, CPPGM_LINKAGE_DEFAULT) ||
		!reserve(symbol, &counts, &buffers))
		return 0;
	buffers.slots[0] = pair_slot();
	buffers.slots[1] = pair_slot();
	buffers.literals[0].low = 8;
	LowirValue eight = value(CPPGM_VALUE_INTEGER, 0);
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_ADDR, 1, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 0));
	code[1] = copy_pair(value(CPPGM_VALUE_TEMP, 0), value(CPPGM_VALUE_TEMP, 1));
	code[2] = unary(CPPGM_OP_LOAD, 2, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 1));
	code[3] = binary(CPPGM_OP_INDEX, 3, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 1), eight);
	code[4] = unary(CPPGM_OP_LOAD, 4, CPPGM_TYPE_F64, value(CPPGM_VALUE_TEMP, 3));
	code[5] = unary(CPPGM_OP_ADDR, 5, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 1));
	code[6] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_F64, value(CPPGM_VALUE_TEMP, 4),
		value(CPPGM_VALUE_TEMP, 5));
	code[7] = binary(CPPGM_OP_INDEX, 6, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 5), eight);
	code[8] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 2),
		value(CPPGM_VALUE_TEMP, 6));
	code[9] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, PAIR, value(CPPGM_VALUE_SLOT, 1));
	return cppgm_lowir_body_commit(module, symbol, &counts, 7);
}

/* function @rg_library_apply(%f : ptr, %skip : f64, %p : obj<16x8> [classes=integer_sse])
 *     -> obj<16x8> [result_classes=sse_integer]
 *   %o = call obj<16x8> %f($s0) as (%a : obj<16x8> [classes=integer_sse])
 *       -> obj<16x8> [result_classes=sse_integer]
 * %skip takes the first SSE register, so the object's double arrives in the
 * second and the call boundary must move it to the first. The call's result
 * is an object temporary, copied to the slot that the function returns. */
static int apply(void)
{
	CppgmParameter incoming[3] = { parameter(CPPGM_TYPE_PTR, 0), parameter(CPPGM_TYPE_F64, 0),
		parameter(PAIR, INTEGER_SSE) };
	CppgmBoundary facts = { 0 };
	facts.result_classes = SSE_INTEGER;
	uint32_t symbol = function(NAME("rg_library_apply"), cppgm_lowir_boundary(module, PAIR, incoming, 3, &facts),
		CPPGM_FUNCTION_DEFINITION);
	uint32_t callee = cppgm_lowir_boundary(module, PAIR, &incoming[2], 1, &facts);
	LowirBodyCounts counts = { 6, 1, 0, 2, 1 };
	LowirBodyBuffers buffers;
	if (callee == CPPGM_NO_ID || !reserve(symbol, &counts, &buffers))
		return 0;
	buffers.slots[0] = pair_slot();
	buffers.slots[1] = pair_slot();
	buffers.operands[0] = value(CPPGM_VALUE_SLOT, 0);
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_ADDR, 3, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 0));
	code[1] = copy_pair(value(CPPGM_VALUE_TEMP, 2), value(CPPGM_VALUE_TEMP, 3));
	code[2] = call(4, PAIR, value(CPPGM_VALUE_TEMP, 0), 0, 1);
	code[2].flags = CPPGM_FLAG_SIGNATURE;
	code[2].immediate = callee;
	code[3] = unary(CPPGM_OP_ADDR, 5, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 1));
	code[4] = copy_pair(value(CPPGM_VALUE_TEMP, 4), value(CPPGM_VALUE_TEMP, 5));
	code[5] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, PAIR, value(CPPGM_VALUE_SLOT, 1));
	return cppgm_lowir_body_commit(module, symbol, &counts, 6);
}

/* declare function @rg_host_tally(%count : i32) -> f64 [arity=variadic]
 * function @rg_library_tally(%n : i64, %d : f64) -> f64
 *   %r = call f64 @rg_host_tally(1, $s0) [classes=1:integer_sse] */
static int tally(void)
{
	CppgmParameter count = parameter(CPPGM_TYPE_I32, 0);
	CppgmBoundary variadic = { 0 };
	variadic.arity = CPPGM_ARITY_VARIADIC;
	uint32_t host = function(NAME("rg_host_tally"), cppgm_lowir_boundary(module, CPPGM_TYPE_F64, &count, 1, &variadic),
		CPPGM_FUNCTION_DECLARATION);
	CppgmType arguments[2] = { CPPGM_TYPE_I64, CPPGM_TYPE_F64 };
	uint32_t symbol = function(NAME("rg_library_tally"),
		cppgm_lowir_signature(module, CPPGM_TYPE_F64, arguments, 2, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 6, 2, 2, 1, 1 };
	LowirBodyBuffers buffers;
	if (host == CPPGM_NO_ID || !reserve(symbol, &counts, &buffers))
		return 0;
	buffers.slots[0] = pair_slot();
	buffers.literals[0].low = 8;
	buffers.literals[1].low = 1;
	buffers.operands[0] = value(CPPGM_VALUE_INTEGER, 1);
	buffers.operands[1] = value(CPPGM_VALUE_SLOT, 0);
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_ADDR, 2, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 0));
	code[1] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 0),
		value(CPPGM_VALUE_TEMP, 2));
	code[2] = binary(CPPGM_OP_INDEX, 3, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 2), value(CPPGM_VALUE_INTEGER, 0));
	code[3] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_F64, value(CPPGM_VALUE_TEMP, 1),
		value(CPPGM_VALUE_TEMP, 3));
	code[4] = call(4, CPPGM_TYPE_F64, value(CPPGM_VALUE_SYMBOL, host), 0, 2);
	code[5] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, CPPGM_TYPE_F64, value(CPPGM_VALUE_TEMP, 4));
	if (!cppgm_lowir_body_commit(module, symbol, &counts, 5))
		return 0;
	/* Facts name the call by its position in the instruction buffer. */
	CppgmArgumentFact fact = { 0 };
	fact.instruction = 4;
	fact.argument = 1;
	fact.classes = INTEGER_SSE;
	return cppgm_lowir_body_argument_facts(module, symbol, &fact, 1);
}

/* assembly @asm0 intel readwrite 1 0 0 2: `movntps [<operand 0>],<operand 1>`.
 * Operand 0 is a captured pointer in r8 to r11. Operand 1 is a v128 whose
 * argument addresses its storage; the backend loads it into xmm0 to xmm3.
 * function @rg_library_stream(%to : ptr, %from : ptr) -> void */
static int stream(void)
{
	static const char text[] = "movntps [],";
	CppgmAsmRegister registers[8] = { { 0 } };
	CppgmAsmGroup groups[8] = { { 0 } };
	for (uint32_t index = 0; index < 8; ++index)
	{
		registers[index].bank = index < 4 ? CPPGM_ASM_BANK_GPR : CPPGM_ASM_BANK_VECTOR;
		registers[index].number = (uint8_t)(index < 4 ? 8 + index : index - 4);
		groups[index].first_register = index;
		groups[index].register_count = 1;
	}
	CppgmAsmAlternative alternatives[2] = { { 0 }, { 0 } };
	CppgmAsmOperand operands[2] = { { 0 }, { 0 } };
	for (uint32_t index = 0; index < 2; ++index)
	{
		alternatives[index].first_group = 4 * index;
		alternatives[index].group_count = 4;
		alternatives[index].matching = CPPGM_NO_ID;
		alternatives[index].locations = CPPGM_ASM_LOCATION_REGISTER;
		operands[index].type = index == 0 ? CPPGM_TYPE_PTR : CPPGM_TYPE_V128;
		operands[index].input = index;
		operands[index].output = CPPGM_NO_ID;
		operands[index].first_alternative = index;
		operands[index].alternative_count = 1;
		operands[index].flags = CPPGM_ASM_OPERAND_READ;
		operands[index].input_storage = (uint8_t)index;
	}
	/* text "movntps [", use 0, text "],", use 1 */
	CppgmAsmPiece pieces[4] = { { 0 }, { 0 }, { 0 }, { 0 } };
	pieces[0].kind = pieces[2].kind = CPPGM_ASM_PIECE_TEXT;
	pieces[0].operand = pieces[2].operand = CPPGM_NO_ID;
	pieces[0].count = 9;
	pieces[2].first = 9;
	pieces[2].count = 2;
	pieces[1].kind = pieces[3].kind = CPPGM_ASM_PIECE_OPERAND;
	pieces[3].operand = 1;
	CppgmAssembly assembly = { 0 };
	assembly.dialect = CPPGM_ASM_DIALECT_INTEL;
	assembly.memory = CPPGM_ASM_MEMORY_READ_WRITE;
	assembly.properties = CPPGM_ASM_PROPERTY_SIDE_EFFECT;
	assembly.argument_count = 2;
	assembly.text = text;
	assembly.text_size = sizeof text - 1;
	assembly.operands = operands;
	assembly.operand_count = 2;
	assembly.alternatives = alternatives;
	assembly.alternative_count = 2;
	assembly.groups = groups;
	assembly.group_count = 8;
	assembly.registers = registers;
	assembly.register_count = 8;
	assembly.pieces = pieces;
	assembly.piece_count = 4;
	uint32_t template = cppgm_lowir_assembly(module, &assembly);
	CppgmType arguments[2] = { CPPGM_TYPE_PTR, CPPGM_TYPE_PTR };
	uint32_t symbol = function(NAME("rg_library_stream"),
		cppgm_lowir_signature(module, CPPGM_TYPE_VOID, arguments, 2, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 2, 2, 0, 0, 1 };
	LowirBodyBuffers buffers;
	if (template == CPPGM_NO_ID || !reserve(symbol, &counts, &buffers))
		return 0;
	buffers.operands[0] = value(CPPGM_VALUE_TEMP, 0);
	buffers.operands[1] = value(CPPGM_VALUE_TEMP, 1);
	LowirInstruction* code = buffers.instructions;
	code[0] = instruction(CPPGM_OP_ASM, CPPGM_NO_ID, CPPGM_TYPE_VOID);
	code[0].immediate = template;
	code[0].extra_count = 2;
	code[1] = instruction(CPPGM_OP_RETURN, CPPGM_NO_ID, CPPGM_TYPE_VOID);
	return cppgm_lowir_body_commit(module, symbol, &counts, 2);
}

/* function @rg_library_turn(%r : obj<16x8> [classes=sse_integer])
 *     -> obj<16x8> [result_classes=integer_sse]
 * The same function as rg_library_swap's kind, stated without classes: its
 * parameter is the record {{f32, f32}, i64}, a nested record, and its result
 * the record {i64, f32[2]}, an array field. Their slots and the return name
 * the records too. The library gives each the classes C gives a struct with
 * those fields: the two floats share an SSE eightbyte. Without the records
 * both eightbytes would be INTEGER. */
static int turn(void)
{
	CppgmField two_fields[2] = { { 0, CPPGM_TYPE_F32, 0 }, { 4, CPPGM_TYPE_F32, 0 } };
	CppgmType two = cppgm_lowir_record(module, 8, 4, two_fields, 2);
	CppgmField floats_fields[2] = { { 0, two, 0 }, { 8, CPPGM_TYPE_I64, 0 } };
	CppgmType floats = two == 0 ? 0 : cppgm_lowir_record(module, 16, 8, floats_fields, 2);
	CppgmField tail_fields[2] = { { 0, CPPGM_TYPE_I64, 0 }, { 8, CPPGM_TYPE_F32, 2 } };
	CppgmType tail = cppgm_lowir_record(module, 16, 8, tail_fields, 2);
	if (floats == 0 || tail == 0)
		return 0;
	uint32_t symbol = function(NAME("rg_library_turn"),
		cppgm_lowir_signature(module, tail, &floats, 1, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 14, 0, 3, 2, 1 };
	LowirBodyBuffers buffers;
	if (!reserve(symbol, &counts, &buffers))
		return 0;
	buffers.slots[0] = pair_slot();
	buffers.slots[0].type = floats;
	buffers.slots[1] = pair_slot();
	buffers.slots[1].type = tail;
	buffers.literals[0].low = 4;
	buffers.literals[1].low = 8;
	buffers.literals[2].low = 12;
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_ADDR, 1, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 0));
	code[1] = copy_pair(value(CPPGM_VALUE_TEMP, 0), value(CPPGM_VALUE_TEMP, 1));
	code[2] = unary(CPPGM_OP_LOAD, 2, CPPGM_TYPE_F32, value(CPPGM_VALUE_TEMP, 1));
	code[3] = binary(CPPGM_OP_INDEX, 3, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 1), value(CPPGM_VALUE_INTEGER, 0));
	code[4] = unary(CPPGM_OP_LOAD, 4, CPPGM_TYPE_F32, value(CPPGM_VALUE_TEMP, 3));
	code[5] = binary(CPPGM_OP_INDEX, 5, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 1), value(CPPGM_VALUE_INTEGER, 1));
	code[6] = unary(CPPGM_OP_LOAD, 6, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 5));
	code[7] = unary(CPPGM_OP_ADDR, 7, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SLOT, 1));
	code[8] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 6),
		value(CPPGM_VALUE_TEMP, 7));
	code[9] = binary(CPPGM_OP_INDEX, 8, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 7), value(CPPGM_VALUE_INTEGER, 1));
	code[10] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_F32, value(CPPGM_VALUE_TEMP, 2),
		value(CPPGM_VALUE_TEMP, 8));
	code[11] = binary(CPPGM_OP_INDEX, 9, CPPGM_TYPE_U8, value(CPPGM_VALUE_TEMP, 7), value(CPPGM_VALUE_INTEGER, 2));
	code[12] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_F32, value(CPPGM_VALUE_TEMP, 4),
		value(CPPGM_VALUE_TEMP, 9));
	code[13] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, tail, value(CPPGM_VALUE_SLOT, 1));
	return cppgm_lowir_body_commit(module, symbol, &counts, 10);
}

/* declare function @exit(%code : i32) -> void [return=noreturn]
 * function @rg_library_stop(%code : i32) -> void [return=noreturn] */
static int stop(void)
{
	CppgmParameter code_parameter = parameter(CPPGM_TYPE_I32, 0);
	CppgmBoundary facts = { 0 };
	facts.returns = CPPGM_RETURN_NORETURN;
	uint32_t signature = cppgm_lowir_boundary(module, CPPGM_TYPE_VOID, &code_parameter, 1, &facts);
	uint32_t host = function(NAME("exit"), signature, CPPGM_FUNCTION_DECLARATION);
	uint32_t symbol = function(NAME("rg_library_stop"), signature, CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 2, 1, 0, 0, 1 };
	LowirBodyBuffers buffers;
	if (host == CPPGM_NO_ID || !reserve(symbol, &counts, &buffers))
		return 0;
	buffers.operands[0] = value(CPPGM_VALUE_TEMP, 0);
	LowirInstruction* code = buffers.instructions;
	code[0] = call(CPPGM_NO_ID, CPPGM_TYPE_VOID, value(CPPGM_VALUE_SYMBOL, host), 0, 1);
	code[1] = instruction(CPPGM_OP_UNREACHABLE, CPPGM_NO_ID, CPPGM_TYPE_VOID);
	return cppgm_lowir_body_commit(module, symbol, &counts, 1);
}

/* function @rg_library_match(%a : v128, %b : v128, %x : i32) -> i32 */
static int match(void)
{
	CppgmType arguments[3] = { CPPGM_TYPE_V128, CPPGM_TYPE_V128, CPPGM_TYPE_I32 };
	uint32_t symbol = function(NAME("rg_library_match"),
		cppgm_lowir_signature(module, CPPGM_TYPE_I32, arguments, 3, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	LowirBodyCounts counts = { 6, 0, 1, 0, 1 };
	LowirBodyBuffers buffers;
	if (!reserve(symbol, &counts, &buffers))
		return 0;
	/* The pshufd immediate is the i32 literal 0 in its third operand. */
	memset(&buffers.literals[0], 0, sizeof buffers.literals[0]);
	LowirInstruction* code = buffers.instructions;
	code[0] = binary(CPPGM_OP_VECTOR, 3, CPPGM_TYPE_V128, value(CPPGM_VALUE_TEMP, 0), value(CPPGM_VALUE_TEMP, 1));
	code[0].sub = CPPGM_VECTOR_PCMPEQB;
	code[1] = unary(CPPGM_OP_VECTOR, 4, CPPGM_TYPE_V128, value(CPPGM_VALUE_TEMP, 2));
	code[1].sub = CPPGM_VECTOR_FROMI32;
	code[2] = unary(CPPGM_OP_VECTOR, 5, CPPGM_TYPE_V128, value(CPPGM_VALUE_TEMP, 4));
	code[2].sub = CPPGM_VECTOR_PSHUFD;
	code[2].ops[2] = value(CPPGM_VALUE_INTEGER, 0);
	code[3] = binary(CPPGM_OP_VECTOR, 6, CPPGM_TYPE_V128, value(CPPGM_VALUE_TEMP, 3), value(CPPGM_VALUE_TEMP, 5));
	code[3].sub = CPPGM_VECTOR_PAND;
	code[4] = unary(CPPGM_OP_VECTOR, 7, CPPGM_TYPE_I32, value(CPPGM_VALUE_TEMP, 6));
	code[4].sub = CPPGM_VECTOR_PMOVMSKB;
	code[5] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, CPPGM_TYPE_I32, value(CPPGM_VALUE_TEMP, 7));
	return cppgm_lowir_body_commit(module, symbol, &counts, 8);
}

/* global @rg_library_ticks [storage=thread_local] = { i64 10 }
 * function @rg_library_tick() -> i64: its thread's instance, incremented. */
static int tick(void)
{
	uint32_t ticks = cppgm_lowir_symbol(module, NAME("rg_library_ticks"));
	uint32_t symbol = function(NAME("rg_library_tick"),
		cppgm_lowir_signature(module, CPPGM_TYPE_I64, NULL, 0, CPPGM_ARITY_FIXED), CPPGM_FUNCTION_DEFINITION);
	CppgmDataItem data = { 0 };
	data.type = CPPGM_TYPE_I64;
	data.bits.low = 10;
	data.kind = CPPGM_DATA_SCALAR;
	LowirBodyCounts counts = { 5, 0, 1, 0, 1 };
	LowirBodyBuffers buffers;
	if (ticks == CPPGM_NO_ID ||
		!cppgm_lowir_global(module, ticks, CPPGM_TYPE_VOID, CPPGM_BINDING_STRONG, CPPGM_STORAGE_THREAD_LOCAL, &data,
			1, CPPGM_GLOBAL_DEFINITION) ||
		!reserve(symbol, &counts, &buffers))
		return 0;
	buffers.literals[0].low = 1;
	LowirInstruction* code = buffers.instructions;
	code[0] = unary(CPPGM_OP_ADDR, 0, CPPGM_TYPE_PTR, value(CPPGM_VALUE_SYMBOL, ticks));
	code[1] = unary(CPPGM_OP_LOAD, 1, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 0));
	code[2] = binary(CPPGM_OP_BINARY, 2, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 1), value(CPPGM_VALUE_INTEGER, 0));
	code[2].sub = CPPGM_BINARY_ADD;
	code[3] = binary(CPPGM_OP_STORE, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 2),
		value(CPPGM_VALUE_TEMP, 0));
	code[4] = unary(CPPGM_OP_RETURN, CPPGM_NO_ID, CPPGM_TYPE_I64, value(CPPGM_VALUE_TEMP, 2));
	return cppgm_lowir_body_commit(module, symbol, &counts, 3);
}

int main(int argc, char** argv)
{
	if (argc != 5 || (strcmp(argv[4], "fixed") != 0 && strcmp(argv[4], "pic") != 0))
		return 2;
	uint32_t form = strcmp(argv[4], "pic") == 0 ? CPPGM_CODE_PIC : CPPGM_CODE_FIXED;
	/* The linked library must be the interface version and release that
	 * the headers and the lock name; any other is refused, not guessed. */
	if (cppgm_lowir_abi_version() != RG_ABI_VERSION || strcmp(cppgm_lowir_version(), argv[3]) != 0)
		return 3;
	/* The library in turn refuses a version outside its range and a target
	 * it does not have. */
	if (cppgm_lowir_create(RG_ABI_VERSION + 1, CPPGM_TARGET_LINUX) != NULL ||
		cppgm_lowir_create(CPPGM_LOWIR_ABI_VERSION_MIN - 1, CPPGM_TARGET_LINUX) != NULL ||
		cppgm_lowir_create(RG_ABI_VERSION, 99) != NULL)
		return 3;
	module = cppgm_lowir_create(RG_ABI_VERSION, CPPGM_TARGET_LINUX);
	if (module == NULL)
		return 3;
	/* The text view must precede compilation, which consumes the module. */
	if (!answer() || !swap() || !apply() || !tally() || !stream() || !turn() || !stop() || !match() || !tick() ||
		!cppgm_lowir_validate(module) || !cppgm_lowir_write(module, argv[2]) ||
		!cppgm_lowir_compile(module, 0, form))
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
