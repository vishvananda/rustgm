#include <stdio.h>
extern unsigned char field_shared(void),field_late_mut(void),field_temporary(void),field_nested(void),field_shadow(void);
void record(unsigned char n){printf("call=%u\n",(unsigned)n);}
int main(void){unsigned char x=field_shared();printf("shared=%u\n",(unsigned)x);x=field_late_mut();printf("late=%u\n",(unsigned)x);x=field_temporary();printf("temporary=%u\n",(unsigned)x);x=field_nested();printf("nested=%u\n",(unsigned)x);x=field_shadow();printf("shadow=%u\n",(unsigned)x);}
