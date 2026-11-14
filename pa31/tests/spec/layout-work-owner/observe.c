#include <stdio.h>
#include <stddef.h>
extern size_t scalar_size(void);
extern size_t pair_size(void);
extern size_t array_size(void);
extern size_t projected_size(void);
extern size_t zero_size(void);
extern size_t pointer_size(void);
extern size_t recursive_size(void);
int main(void){printf("%zu %zu %zu %zu %zu %zu %zu\n",scalar_size(),pair_size(),array_size(),projected_size(),zero_size(),pointer_size(),recursive_size());return 0;}
