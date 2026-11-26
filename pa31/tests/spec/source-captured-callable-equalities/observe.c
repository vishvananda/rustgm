#include <stdint.h>
#include <stdio.h>
uint32_t obs_nested(void),obs_referent(void),obs_choose(void),obs_changed(void),obs_kept(void),obs_free_value(void),obs_free_capture(void);
int main(void) {
    printf("%u %u %u %u %u %u %u\n",obs_nested(),obs_referent(),obs_choose(),obs_changed(),obs_kept(),obs_free_value(),obs_free_capture());
    return 0;
}
