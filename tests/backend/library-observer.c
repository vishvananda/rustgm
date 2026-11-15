/* Original RustGM maintainer input: an independent C caller of the object
 * that library.c compiled in process. The host compiler places each argument
 * and result by the psABI, so a check fails when the boundary fact it names
 * did not reach the object. */
#include <stdarg.h>
#include <stdint.h>
#include <string.h>
#include <xmmintrin.h>

/* Eightbyte classes INTEGER, SSE and SSE, INTEGER. */
struct rg_first {
	long n;
	double d;
};
struct rg_second {
	double d;
	long n;
};

extern const long rg_library_seed;
long rg_library_answer(long);
struct rg_second rg_library_swap(struct rg_first);
struct rg_second rg_library_apply(struct rg_second (*)(struct rg_first), double, struct rg_first);
double rg_library_tally(long, double);
void rg_library_stream(float*, const __m128*);
/* Not declared noreturn here: a return must reach the failure below. */
void rg_library_stop(int);

static struct rg_second turn(struct rg_first value)
{
	struct rg_second result = { value.d * 2.0, value.n + 1 };
	return result;
}

double rg_host_tally(int count, ...)
{
	va_list arguments;
	va_start(arguments, count);
	struct rg_first value = va_arg(arguments, struct rg_first);
	va_end(arguments);
	return (double)count + value.d * (double)value.n;
}

struct __attribute__((aligned(16))) storage {
	uint64_t before[2];
	uint32_t bits[4];
	uint64_t after[2];
};

/* Every bit of the vector reaches the aligned destination and no neighbour. */
static int streamed(void)
{
	static const uint32_t pattern[4] = { 0x80000000u, 0x7fc12345u, 0x00000001u, 0xff800000u };
	struct storage out = { { 0x0123456789abcdefull, 0xfedcba9876543210ull },
		{ 0, 0, 0, 0 },
		{ 0x55aa55aa55aa55aaull, 0xaa55aa55aa55aa55ull } };
	__m128 vector;
	memcpy(&vector, pattern, sizeof vector);
	rg_library_stream((float*)out.bits, &vector);
	_mm_sfence();
	return memcmp(out.bits, pattern, sizeof pattern) == 0 && out.before[0] == 0x0123456789abcdefull &&
		out.before[1] == 0xfedcba9876543210ull && out.after[0] == 0x55aa55aa55aa55aaull &&
		out.after[1] == 0xaa55aa55aa55aa55ull;
}

int main(void)
{
	if (rg_library_seed != 40)
		return 1;
	if (rg_library_answer(2) != 42)
		return 2;
	if (rg_library_answer(-40) != 0)
		return 3;
	struct rg_first first = { 7, 2.5 };
	struct rg_second second = rg_library_swap(first);
	if (second.d != 2.5 || second.n != 7)
		return 4;
	first.n = 4;
	first.d = 10.5;
	second = rg_library_apply(turn, 99.0, first);
	if (second.d != 21.0 || second.n != 5)
		return 5;
	if (rg_library_tally(6, 3.5) != 22.0)
		return 6;
	if (!streamed())
		return 7;
	/* Success is the exit status that the noreturn function passes on. */
	rg_library_stop(0);
	return 8;
}
