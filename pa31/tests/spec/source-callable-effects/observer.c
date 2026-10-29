#include <stdint.h>
#include <stdio.h>
extern uint8_t const_result(void),unsafe_result(void),combined_result(void),generic_result(void),forward_result(void),const_forward_result(void);
int main(void){printf("%u %u %u %u %u %u\n",const_result(),unsafe_result(),combined_result(),generic_result(),forward_result(),const_forward_result());return 0;}
