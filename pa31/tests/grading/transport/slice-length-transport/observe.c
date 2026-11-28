#include <stddef.h>
extern size_t observed_const(void);
extern size_t observed_runtime(void);
int main(void) { return observed_const() == 3 && observed_runtime() == 5 ? 0 : 1; }
