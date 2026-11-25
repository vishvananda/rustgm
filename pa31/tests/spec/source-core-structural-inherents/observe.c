#include <stdint.h>
#include <stdio.h>

extern uint32_t observed_unit_const(void);
extern uint32_t observed_unit_method(void);
extern uint32_t observed_tuple_method(void);
extern uint32_t observed_ref_method(void);
extern uint32_t observed_mut_ref_method(void);
extern uint32_t observed_fn_ptr(void);

int main(void) {
    printf("%u %u %u %u %u %u\n",
           observed_unit_const(),
           observed_unit_method(),
           observed_tuple_method(),
           observed_ref_method(),
           observed_mut_ref_method(),
           observed_fn_ptr());
    return 0;
}
