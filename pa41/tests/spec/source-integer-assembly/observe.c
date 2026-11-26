#include <stdint.h>
#include <stdio.h>
uint64_t obs_increment(void), obs_late(void), obs_aliased(void), obs_stored(void);
int64_t obs_att(void);
int main(void) { printf("%llu %llu %llu %llu %lld\n", (unsigned long long)obs_increment(), (unsigned long long)obs_late(), (unsigned long long)obs_aliased(), (unsigned long long)obs_stored(), (long long)obs_att()); return 0; }
