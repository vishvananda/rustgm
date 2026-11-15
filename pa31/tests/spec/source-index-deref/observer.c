#include <stdio.h>
typedef unsigned char u8;
extern u8 index_shared(u8,u8,u8),index_nested(u8,u8,u8),index_reference(u8,u8,u8),index_double(u8,u8,u8),index_slice(u8,u8,u8),index_custom(u8,u8,u8),index_own(u8,u8,u8);
extern u8 index_store(u8,u8,u8,u8),index_compound(u8,u8,u8,u8),index_borrow(u8,u8,u8,u8),index_nested_store(u8,u8,u8,u8),index_reference_store(u8,u8,u8,u8),index_late(u8,u8,u8),index_generic(u8,u8,u8);
void record(u8 n){printf("call=%u\n",(unsigned)n);}
int main(void){volatile u8 a=11,b=22,c=33,v=64;unsigned x;
x=index_shared(a,b,c);printf("shared=%u\n",x);
x=index_nested(a,b,c);printf("nested=%u\n",x);
x=index_reference(a,b,c);printf("reference=%u\n",x);
x=index_double(a,b,c);printf("double=%u\n",x);
x=index_slice(a,b,c);printf("slice=%u\n",x);
x=index_custom(a,b,c);printf("custom=%u\n",x);
x=index_own(a,b,c);printf("own=%u\n",x);
x=index_store(a,b,c,v);printf("store=%u\n",x);
x=index_compound(a,b,c,v);printf("compound=%u\n",x);
x=index_borrow(a,b,c,v);printf("borrow=%u\n",x);
x=index_nested_store(a,b,c,v);printf("nested_store=%u\n",x);
x=index_reference_store(a,b,c,v);printf("reference_store=%u\n",x);
x=index_late(a,b,v);printf("late=%u\n",x);
x=index_generic(a,b,c);printf("generic=%u\n",x);
return 0;}
