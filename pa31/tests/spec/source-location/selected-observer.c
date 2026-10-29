#include <stdint.h>
#include <stdio.h>
extern uint32_t selected_runtime_line(void),selected_constant_line(void),untracked_rhs(void),untracked_self(void);
int main(void){uint32_t a=selected_runtime_line(),b=selected_constant_line(),c=untracked_rhs(),d=untracked_self();printf("%u %u %u %u\n",a,b,c,d);return a!=372||b!=375||c!=73||d!=89;}
