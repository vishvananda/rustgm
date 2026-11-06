#include <stdio.h>
extern const signed char *rebuild(const signed char*),*safe_build(void);
int main(void){const signed char input[3]={111,107,0};const signed char*p=rebuild(input),*q=safe_build();if(p!=input||p[0]!=111||p[1]!=107||p[2]||q[0]!=111||q[1]!=107||q[2])return 1;printf("construct %d %d %d\n",p[0],p[1],p[2]);return 0;}
