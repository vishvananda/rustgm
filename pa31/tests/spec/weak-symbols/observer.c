#include <stddef.h>
#include <stdio.h>
extern void *call_memcpy(void *,const void *,size_t);
extern void *call_memmove(void *,const void *,size_t);
extern void *call_memset(void *,int,size_t);
extern int call_memcmp(const void *,const void *,size_t);
extern int call_bcmp(const void *,const void *,size_t);
extern size_t call_strlen(const signed char *);
int main(void){unsigned char a[6]={1,2,3,4,5,0},b[6]={0};
int c=call_memcpy(b,a,6)==b && b[4]==5;
int m=call_memmove(b+1,b,4)==b+1 && b[4]==4;
int s=call_memset(b+1,7,2)==b+1 && b[1]==7 && b[2]==7;
int d=call_memcmp(a,b,6)<0;int e=call_bcmp(a,a,6)==0;
size_t n=call_strlen((const signed char *)"hello");
printf("runtime %d %d %d %d %d %zu\n",c,m,s,d,e,n);
return !(c&&m&&s&&d&&e&&n==5);}
