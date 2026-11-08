#include <stdint.h>
#include <string.h>
#include <xmmintrin.h>

extern void rg_stream_ps(float *, __m128);
extern void rg_stream_ps_stored(float *, const __m128 *);

struct __attribute__((aligned(16))) storage {
    uint64_t before[2];
    uint32_t bits[4];
    uint64_t after[2];
};

static int streamed(const uint32_t pattern[4], int stored) {
    struct storage out = {{0x0123456789abcdefull, 0xfedcba9876543210ull},
                          {0, 0, 0, 0},
                          {0x55aa55aa55aa55aaull, 0xaa55aa55aa55aa55ull}};
    __m128 value;
    memcpy(&value, pattern, sizeof value);
    if (stored) {
        rg_stream_ps_stored((float *)out.bits, &value);
    } else {
        rg_stream_ps((float *)out.bits, value);
    }
    _mm_sfence();
    return ((uintptr_t)out.bits & 15u) == 0 && memcmp(out.bits, pattern, 16) == 0 &&
           out.before[0] == 0x0123456789abcdefull && out.before[1] == 0xfedcba9876543210ull &&
           out.after[0] == 0x55aa55aa55aa55aaull && out.after[1] == 0xaa55aa55aa55aa55ull;
}

int main(void) {
    static const uint32_t patterns[][4] = {
        {0x80000000u, 0x7fc12345u, 0x00000001u, 0xff800000u},
        {0x11223344u, 0x55667788u, 0x99aabbccu, 0xddeeff00u},
        {0xffffffffu, 0x00000000u, 0x7f800000u, 0x007fffffu},
    };
    for (unsigned index = 0; index < sizeof patterns / sizeof patterns[0]; ++index) {
        if (!streamed(patterns[index], 0) || !streamed(patterns[index], 1)) {
            return 1;
        }
    }
    return 0;
}
