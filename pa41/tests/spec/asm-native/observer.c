#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <emmintrin.h>

/* Each destination is 16 aligned bytes between two 16-byte sentinels. */
typedef struct {
    _Alignas(16) uint32_t words[12];
} Frame;

void rg_stream_ps(float *p, __m128 a);
void rg_stream_lanes(float *p);
void rg_stream_default(float *p, __m128 a);
void rg_reassigned(float *p, float *q);
void rg_operand_order(float *p, float *q);
void rg_generic(float *p, float *q, __m128 a, __m128 b);

/* A quiet NaN payload, -0, the least denormal, +inf, all ones, a negative
   signaling NaN, 1 and the least normal. */
static const uint32_t bits[8] = {
    0x7fc12345u, 0x80000000u, 0x00000001u, 0x7f800000u,
    0xffffffffu, 0xff800001u, 0x3f800000u, 0x00800000u,
};
static const uint32_t guard[8] = {
    0x01234567u, 0x89abcdefu, 0xfedcba98u, 0x76543210u,
    0xa5a5a5a5u, 0x5a5a5a5au, 0xc3c3c3c3u, 0x3c3c3c3cu,
};

float lane(uint32_t index) {
    float value;
    memcpy(&value, &bits[index & 7u], sizeof value);
    return value;
}

static float *arm(Frame *frame) {
    for (int i = 0; i < 4; ++i) {
        frame->words[i] = guard[i];
        frame->words[4 + i] = 0x55555555u;
        frame->words[8 + i] = guard[4 + i];
    }
    if ((uintptr_t)(frame->words + 4) % 16u != 0) {
        fputs("destination is not 16-byte aligned\n", stderr);
        exit(2);
    }
    return (float *)(frame->words + 4);
}

static __m128 vector(int first) {
    return _mm_castsi128_ps(_mm_setr_epi32((int)bits[first], (int)bits[first + 1],
                                           (int)bits[first + 2], (int)bits[first + 3]));
}

/* Print the destination's bits; a sentinel change stops the observer. */
static void report(const Frame *frame) {
    for (int i = 0; i < 4; ++i) {
        if (frame->words[i] != guard[i] || frame->words[8 + i] != guard[4 + i]) {
            fputs("sentinel changed\n", stderr);
            exit(3);
        }
    }
    printf("%08x %08x %08x %08x", frame->words[4], frame->words[5], frame->words[6],
           frame->words[7]);
}

static void single(const char *name, const Frame *frame) {
    printf("%s: ", name);
    report(frame);
    putchar('\n');
}

static void pair(const char *name, const Frame *first, const Frame *second) {
    printf("%s: ", name);
    report(first);
    fputs(" | ", stdout);
    report(second);
    putchar('\n');
}

void rg_observe(void) {
    Frame a, b;
    rg_stream_ps(arm(&a), vector(0));
    _mm_sfence();
    single("original by value", &a);
    rg_stream_lanes(arm(&a));
    _mm_sfence();
    single("original from lanes", &a);
    rg_stream_default(arm(&a), vector(4));
    _mm_sfence();
    single("default options", &a);
    rg_reassigned(arm(&a), arm(&b));
    _mm_sfence();
    pair("reassigned after asm", &a, &b);
    rg_operand_order(arm(&a), arm(&b));
    _mm_sfence();
    pair("assigned by a later operand", &a, &b);
    rg_generic(arm(&a), arm(&b), vector(4), vector(0));
    _mm_sfence();
    pair("generic instances", &a, &b);
}
