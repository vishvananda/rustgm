#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

extern uint16_t obs_half_low(void), obs_half_tiny(void), obs_half_negative_zero(void);
extern uint64_t obs_quad_low_low(void), obs_quad_low_high(void);
extern uint64_t obs_quad_pi_low(void), obs_quad_pi_high(void);
extern uint64_t obs_quad_negative_zero_low(void), obs_quad_negative_zero_high(void);
extern uint16_t obs_half_generic(void);
extern uint64_t obs_quad_generic(void);

int main(void) {
    printf("%04x %04x %04x %016" PRIx64 " %016" PRIx64 " %016" PRIx64
           " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %04x %016" PRIx64 "\n",
           (unsigned)obs_half_low(), (unsigned)obs_half_tiny(), (unsigned)obs_half_negative_zero(),
           obs_quad_low_low(), obs_quad_low_high(), obs_quad_pi_low(), obs_quad_pi_high(),
           obs_quad_negative_zero_low(), obs_quad_negative_zero_high(),
           (unsigned)obs_half_generic(), obs_quad_generic());
    return 0;
}
