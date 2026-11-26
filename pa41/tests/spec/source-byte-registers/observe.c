#include <stdint.h>
#include <stdio.h>

#define DECLARE(N, T) \
    void observe_bittest##N(uintptr_t, T, uintptr_t); \
    void observe_bittestandset##N(uintptr_t, T, uintptr_t); \
    void observe_bittestandreset##N(uintptr_t, T, uintptr_t); \
    void observe_bittestandcomplement##N(uintptr_t, T, uintptr_t);

DECLARE(32, int32_t)
DECLARE(64, int64_t)

#define CHECK(call, expected, first, second) do { \
    result[1] = 0xff; \
    call; \
    if (result[0] != 71 || result[1] != (expected) || result[2] != 91 || \
        words[0] != (first) || words[1] != (second) || words[2] != 37) return 1; \
} while (0)

#define TEST(N, T) do { \
    T words[] = {16, 2, 37}; \
    const T high = (T)((uint##N##_t)1 << (N - 1)); \
    CHECK(observe_bittest##N((uintptr_t)words, 4, (uintptr_t)(result + 1)), 1, 16, 2); \
    CHECK(observe_bittest##N((uintptr_t)words, N, (uintptr_t)(result + 1)), 0, 16, 2); \
    CHECK(observe_bittest##N((uintptr_t)words, N + 1, (uintptr_t)(result + 1)), 1, 16, 2); \
    CHECK(observe_bittestandset##N((uintptr_t)(words + 1), -1, (uintptr_t)(result + 1)), 0, 16 | high, 2); \
    CHECK(observe_bittestandreset##N((uintptr_t)(words + 1), -1, (uintptr_t)(result + 1)), 1, 16, 2); \
    CHECK(observe_bittestandcomplement##N((uintptr_t)words, N + 1, (uintptr_t)(result + 1)), 1, 16, 0); \
    CHECK(observe_bittestandcomplement##N((uintptr_t)words, N + 1, (uintptr_t)(result + 1)), 0, 16, 2); \
} while (0)

int main(void) {
    uint8_t result[] = {71, 0, 91};
    TEST(32, int32_t);
    TEST(64, int64_t);
    puts("bittest 32/64 passed");
    return 0;
}
