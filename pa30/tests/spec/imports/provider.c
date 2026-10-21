#include <stdint.h>
int32_t imported_i32(int32_t x) { return x + 3; }
double imported_f64(double x) { return x + 2.5; }
int32_t invoke_callback(int32_t (*callback)(int32_t), int32_t x) {
    return callback(x) + 2;
}
