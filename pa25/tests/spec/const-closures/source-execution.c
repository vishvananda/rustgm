#include <stdint.h>
#include <stdio.h>
extern uintptr_t observed_count(void);
extern uintptr_t observed_capture(void);
extern uintptr_t observed_runtime(uintptr_t);
int main(void) {
    uintptr_t count=observed_count(),capture=observed_capture(),runtime=observed_runtime(23);
    printf("closure %lu %lu %lu\n",(unsigned long)count,(unsigned long)capture,(unsigned long)runtime);
    return count!=2 || capture!=23 || runtime!=25;
}
