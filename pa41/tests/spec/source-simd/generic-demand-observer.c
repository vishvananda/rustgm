#include <stdio.h>
#include <stddef.h>
extern size_t scalar_lane_layout(void);
extern size_t bool_lane_layout(void);
int main(void) { printf("%zu %zu\n",scalar_lane_layout(),bool_lane_layout()); }
