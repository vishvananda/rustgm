#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
struct Record{uint8_t byte;uint32_t word;};
static unsigned calls;
uint32_t field_value(void){++calls;return 31;}
extern size_t field_offset_c(void);
extern uint32_t field_c(uint8_t,uint32_t),field_rust(uint8_t,uint32_t),field_tuple(uint8_t,uint32_t),field_generic(uint32_t),field_effect(void);
extern uint8_t field_borrowed(const uint8_t*);
int main(void){uint8_t value=47;size_t offset=field_offset_c();uint32_t c=field_c(17,23),r=field_rust(19,29),t=field_tuple(7,37),g=field_generic(41),e=field_effect();unsigned borrowed=field_borrowed(&value);if(offset!=offsetof(struct Record,word)||c!=23||r!=29||t!=37||g!=41||borrowed!=47||e!=31||calls!=1)return 1;printf("field %zu %u %u %u %u %u %u %u\n",offset,c,r,t,g,borrowed,e,calls);return 0;}
