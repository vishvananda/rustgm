#include <stdint.h>
#include <stdio.h>

extern uint32_t routed_line(void);
extern uint32_t alternate_line(void);
extern uint32_t nested_line(void);
extern uint32_t forwarded(void);
extern uint32_t opaque_at_provider(void);
extern uint32_t transparent_at_client(void);

int main(void) {
    printf("%u %u %u %u %u %u\n", (unsigned)routed_line(), (unsigned)alternate_line(),
           (unsigned)nested_line(), (unsigned)forwarded(), (unsigned)opaque_at_provider(),
           (unsigned)transparent_at_client());
    return 0;
}
