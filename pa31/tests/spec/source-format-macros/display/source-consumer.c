#include <stdint.h>
#include <stdio.h>
#include <string.h>
extern uint32_t answer(void);
extern uintptr_t expected_debug(void);
extern uintptr_t expected_display(void);
uint32_t tick(uint32_t which){printf("e%u;",which);return which==1?7:9;}
void gone(uint32_t value){printf("d%u;",value);}
void read_seen(uint32_t a,uint32_t b){(void)a;(void)b;}
void debug_fmt(void){printf("CALLED-D;");}
void display_fmt(void){printf("CALLED-M;");}
void argument_seen(uint32_t slot,uint32_t value,uintptr_t formatter){
 char kind=formatter==expected_debug()?'D':formatter==expected_display()?'M':'?';
 printf("p%u%c%u;",slot,kind,value);
}
void template_seen(const unsigned char *bytes){
 static const unsigned char expected[]={0xc0,1,'|',0xc0,1,'|',0xc8,0,0,1,'|',0xc8,2,0,1,'|',0xc8,1,0,0};
 printf("t%d;",memcmp(bytes,expected,sizeof expected)==0);
}
int main(void){uint32_t count=answer();printf("%u\n",count);return count==3?0:1;}
