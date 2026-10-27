#include <stdlib.h>
int rustgm_simd_probe(void);
void rustgm_observe(void) { int status = rustgm_simd_probe(); if (status != 0) exit(status); }
