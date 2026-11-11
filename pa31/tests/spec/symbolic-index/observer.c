#include <stdio.h>
extern int observed_read(int,int),observed_explicit(int,int),observed_write(int,int,int),observed_left(int,int,int),observed_raw(int,int),observed_raw_mut(int,int,int),observed_custom(int,int);
extern int observed_const(void);
int main(void){volatile int a=23,b=41,n=47;int r=observed_read(a,b),e=observed_explicit(a,b),w=observed_write(a,b,n),l=observed_left(a,b,n),p=observed_raw(a,b),m=observed_raw_mut(a,b,n),k=observed_custom(a,b),v=observed_const();printf("symbolic-index %d %d %d %d %d %d %d %d\n",r,e,w,l,p,m,k,v);return r!=41||e!=23||w!=47||l!=23||p!=41||m!=23||k!=23||v!=47;}
