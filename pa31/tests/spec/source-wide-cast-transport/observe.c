#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
extern uint32_t obs_single(void);
extern uint64_t obs_double(void);
extern uint16_t obs_half_tie(void);
extern uint16_t obs_half_round(void);
extern uint32_t obs_quad_round_single(void);
extern uint64_t obs_quad_round_double(void);
extern uint64_t obs_double_quad_low(void);
extern uint64_t obs_double_quad_high(void);
extern uint64_t obs_half_quad_low(void);
extern uint64_t obs_half_quad_high(void);
extern uint16_t obs_quad_half(void);
extern int32_t obs_signed(void);
extern int32_t obs_nan(void);
extern uint8_t obs_saturated(void);
extern uint16_t obs_integer_half(void);
extern uint64_t obs_integer_quad_low(void);
extern uint64_t obs_integer_quad_high(void);
extern uint64_t obs_quad_integer_low(void);
extern uint64_t obs_quad_integer_high(void);
extern uint32_t obs_generic(void);
extern int32_t obs_positive_inf(void);
extern int32_t obs_negative_inf(void);
extern uint8_t obs_negative_byte(void);
extern uint16_t obs_signed_integer_half(void);
extern uint64_t obs_quad_signed_low(void);
extern uint64_t obs_quad_signed_high(void);
extern uint64_t obs_quad_saturate_low(void);
extern uint64_t obs_quad_saturate_high(void);
extern uint16_t obs_subnormal_tie(void);
extern uint16_t obs_subnormal_above(void);
extern uint16_t obs_negative_half(void);
extern uint64_t obs_negative_zero_low(void);
extern uint64_t obs_negative_zero_high(void);
int main(void) {
    printf("%08" PRIx32 " %016" PRIx64 " %04x %04x %08" PRIx32 " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %04x %" PRId32 " %" PRId32 " %u %04x %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %08" PRIx32 " %" PRId32 " %" PRId32 " %u %04x %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %016" PRIx64 " %04x %04x %04x %016" PRIx64 " %016" PRIx64 "\n",
           obs_single(),
           obs_double(),
           (unsigned)obs_half_tie(),
           (unsigned)obs_half_round(),
           obs_quad_round_single(),
           obs_quad_round_double(),
           obs_double_quad_low(),
           obs_double_quad_high(),
           obs_half_quad_low(),
           obs_half_quad_high(),
           (unsigned)obs_quad_half(),
           obs_signed(),
           obs_nan(),
           (unsigned)obs_saturated(),
           (unsigned)obs_integer_half(),
           obs_integer_quad_low(),
           obs_integer_quad_high(),
           obs_quad_integer_low(),
           obs_quad_integer_high(),
           obs_generic(),
           obs_positive_inf(),
           obs_negative_inf(),
           (unsigned)obs_negative_byte(),
           (unsigned)obs_signed_integer_half(),
           obs_quad_signed_low(),
           obs_quad_signed_high(),
           obs_quad_saturate_low(),
           obs_quad_saturate_high(),
           (unsigned)obs_subnormal_tie(),
           (unsigned)obs_subnormal_above(),
           (unsigned)obs_negative_half(),
           obs_negative_zero_low(),
           obs_negative_zero_high());
    return 0;
}
