#include <stdint.h>
extern uint32_t (*const ffi_callback)(uint32_t);
uint32_t invoke_exported(uint32_t x){return ffi_callback(x);}
