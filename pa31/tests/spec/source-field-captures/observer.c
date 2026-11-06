#include <stdio.h>
extern unsigned char cap_shared(void),cap_mutable(void),cap_move_read(void),cap_once(void),cap_disjoint(void),cap_temporary(void),cap_nested(void),cap_reference(void);
extern void cap_unpolled(void);
void record(unsigned char value){printf("event=%u\n",(unsigned)value);}
int main(void){puts("shared");printf("value=%u\n",(unsigned)cap_shared());puts("mutable");printf("value=%u\n",(unsigned)cap_mutable());puts("move-read");printf("value=%u\n",(unsigned)cap_move_read());puts("once");printf("value=%u\n",(unsigned)cap_once());puts("unpolled");cap_unpolled();puts("disjoint");printf("value=%u\n",(unsigned)cap_disjoint());puts("temporary");printf("value=%u\n",(unsigned)cap_temporary());puts("nested");printf("value=%u\n",(unsigned)cap_nested());puts("reference");printf("value=%u\n",(unsigned)cap_reference());return 0;}
