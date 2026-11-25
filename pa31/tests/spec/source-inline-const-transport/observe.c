#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_ordinary(void);
extern uint32_t obs_const(void);
extern uint32_t obs_generic(void);
extern uint32_t obs_closure(void);
extern uint32_t obs_nested(void);
extern uint32_t obs_initializer(void);

extern uint32_t obs_type_const(void);
extern uint32_t obs_type_runtime(void);

int main(void) {
    printf("%u %u %u %u %u %u %u %u\n", obs_ordinary(), obs_const(), obs_generic(),
           obs_closure(), obs_nested(), obs_initializer(), obs_type_const(), obs_type_runtime());
    return 0;
}
