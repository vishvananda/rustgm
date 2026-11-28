#include <stdint.h>
extern uint8_t observed_field(void);
extern uint8_t observed_signature(void);
extern uint8_t observed_higher_ranked(void);
int main(void) { return observed_field() == 41 && observed_signature() == 7 && observed_higher_ranked() == 23 ? 0 : 1; }
