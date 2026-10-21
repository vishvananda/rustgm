#include <stdint.h>
struct Pair { uint32_t a; double b; };
extern const uint32_t ffi_readonly;
extern uint32_t ffi_mutable;
extern const struct Pair ffi_pair;
uint32_t read_exports(void){return ffi_readonly+ffi_mutable;}
void change_export(void){ffi_mutable+=3;}
double read_pair(void){return ffi_pair.a+ffi_pair.b;}
