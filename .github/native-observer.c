#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
extern uint32_t float_bits(float);
extern uint32_t signed_bits(int32_t);
extern uint32_t const_bits(void);
extern uintptr_t tagged_numeric(void);
extern uint8_t pointer_roundtrip(void);
extern uint8_t function_roundtrip(void);
extern uint8_t moved(uint8_t);
extern uint8_t same_name(void);
extern uint8_t identity_value(void);
void source_dropped(void){puts("source_drop");}
void destination_dropped(void){puts("destination_drop");}
int main(void){
 uint8_t result=moved(19);
 printf("%08" PRIx32 " %08" PRIx32 " %08" PRIx32 " %" PRIuPTR " %u %u %u %u %u\n",
 float_bits(1.0f),signed_bits(-1),const_bits(),tagged_numeric(),
 (unsigned)pointer_roundtrip(),(unsigned)function_roundtrip(),(unsigned)result,
 (unsigned)same_name(),(unsigned)identity_value());
 return 0;
}
