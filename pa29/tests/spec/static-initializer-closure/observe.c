#include <stdint.h>

extern unsigned char call_handler(unsigned char);
extern uintptr_t address(void);

int main(void) {
    uintptr_t first = address();
    return first && first == address() && call_handler(5) == 2
        && call_handler(15) == 8 ? 0 : 1;
}
