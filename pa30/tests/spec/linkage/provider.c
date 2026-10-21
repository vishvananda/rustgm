#include <stdint.h>
extern int32_t ffi_named(int32_t);
extern int32_t ffi_renamed(int32_t);
extern int32_t ffi_macro(int32_t);
extern int32_t ffi_cfg(int32_t);
extern int32_t special(int32_t) __asm__("ffi.$0");
extern double ffi_float(double);
int32_t call_named(int32_t x) { return ffi_named(x); }
int32_t call_renamed(int32_t x) { return ffi_renamed(x); }
int32_t call_macro(int32_t x) { return ffi_macro(x); }
int32_t call_cfg(int32_t x) { return ffi_cfg(x); }
int32_t call_special(int32_t x) { return special(x); }
double call_float(double x) { return ffi_float(x); }
