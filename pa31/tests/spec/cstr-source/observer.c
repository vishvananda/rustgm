#include <stdint.h>
#include <stdio.h>
extern const signed char *ordinary(void), *empty(void), *utf8(void), *high(void), *raw(void), *quoted(void);
extern const unsigned char *bytes(void);
extern uint8_t size(void), empty_size(void);
int main(void){const unsigned char*a=(const unsigned char*)ordinary(),*u=(const unsigned char*)utf8(),*q=(const unsigned char*)quoted(),*r=(const unsigned char*)raw();if(a[0]!=65||a[1]||empty()[0]||u[0]!=195||u[1]!=169||u[2]||(unsigned char)high()[0]!=255||high()[1]||r[0]!=65||r[1]!=92||r[2]!=110||r[3]||q[0]!=97||q[1]!=34||q[2]!=35||q[3]!=98||q[4]||bytes()!=a||size()!=2||empty_size()!=1)return 1;printf("CStr %u %u %u %u %u %u %u %u\n",a[0],size(),empty()[0],u[0],u[1],(unsigned char)high()[0],r[1],empty_size());return 0;}
