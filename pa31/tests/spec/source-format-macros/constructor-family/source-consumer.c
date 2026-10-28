#include <stdint.h>
#include <stdio.h>
#include <string.h>
extern uint32_t answer(void);
extern uintptr_t expected_formatter(uint32_t);
extern uintptr_t expected_debug(void),expected_display(void),expected_lower_hex(void);
uint32_t tick(uint32_t which){printf("e%u;",which);return which==1?7:9;}
void gone(uint32_t value){printf("d%u;",value);}
void read_seen(uint32_t a,uint32_t b){(void)a;(void)b;}
void debug_fmt(void){printf("CALLED-D;");}
void display_fmt(void){printf("CALLED-M;");}
void lower_hex_fmt(void){printf("CALLED-L;");}
void argument_seen(uint32_t slot,uint32_t value,uintptr_t formatter){
 const char tags[]="MDOLUPBeE";char kind='?';
 for(uint32_t i=0;i<9;i++){if(formatter==expected_formatter(i)){kind=tags[i];break;}}
 printf("p%u%c%u;",slot,kind,value);
}
void template_seen(const unsigned char *bytes){
 static const unsigned char expected[]={0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,1,'|',0xc0,0};
 printf("t%d;",memcmp(bytes,expected,sizeof expected)==0);
}
int main(void){uint32_t count=answer();printf("%u\n",count);return count==9?0:1;}
