#include <stdint.h>
#include <stdio.h>
typedef int64_t V __attribute__((vector_size(16)));extern V clone_vector(V);int main(void){V v={INT64_C(0x123456789abcdef),-17};V r=clone_vector(v);if(r[0]!=v[0]||r[1]!=v[1])return 1;puts("simd copy clone 128 bits");return 0;}
