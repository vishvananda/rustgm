#include <stdint.h>
#include <stdio.h>
uint64_t obs_flags(void), obs_increment(void), obs_late(void), obs_aliased(void), obs_stored(void);
int64_t obs_att(void);
int main(void) { printf("%u %llu %llu %llu %llu %lld\n", (unsigned)((obs_flags() >> 1) & 1), (unsigned long long)obs_increment(), (unsigned long long)obs_late(), (unsigned long long)obs_aliased(), (unsigned long long)obs_stored(), (long long)obs_att()); return 0; }
