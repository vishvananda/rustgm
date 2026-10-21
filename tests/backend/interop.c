#include <stdint.h>

extern int32_t rg_add(int32_t, int32_t);
extern double rg_fp(double, double);
extern int32_t rg_import(int32_t);
extern int32_t rg_callback(int32_t (*)(int32_t), int32_t);
extern int32_t rg_read_global(void);
extern int32_t rg_export_global;

int32_t host_global = 93;
int32_t host_add(int32_t value) { return value + 7; }

int main(void) {
    return !(rg_add(-20, 62) == 42 && rg_fp(1.25, 2.5) == 3.75 &&
             rg_import(35) == 42 && rg_callback(host_add, 35) == 42 &&
             rg_read_global() == 93 && rg_export_global == 17);
}
