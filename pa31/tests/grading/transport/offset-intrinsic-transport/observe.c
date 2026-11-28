#include <stddef.h>
extern size_t observed_const(void);
extern size_t observed_runtime(void);
int main(void) { return observed_const() == 8 && observed_runtime() == 16 ? 0 : 1; }
