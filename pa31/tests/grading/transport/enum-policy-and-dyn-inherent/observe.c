#include <stdint.h>
extern uint8_t observed_void(void);
extern uint8_t observed_dyn_inherent(void);
extern uint8_t observed_dyn_send_inherent(void);
int main(void) { return observed_void() == 41 && observed_dyn_inherent() == 7 && observed_dyn_send_inherent() == 23 ? 0 : 1; }
