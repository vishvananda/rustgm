#include <stddef.h>
#include <stdio.h>
extern const unsigned char *byte_const(void),*byte_static(void),*c_view(void);
extern const signed char *c_const(void),*c_static(void);
extern size_t lengths(void),const_width(void);
extern const signed char *const_pointer(void);
extern const unsigned char *const_view(void),*static_view(void),*high_view(void);
int main(void){const unsigned char*a=byte_const(),*b=byte_static(),*c=(const unsigned char*)c_const(),*d=(const unsigned char*)c_static();if(const_width()!=2||const_pointer()[0]!=65||const_pointer()[1]||const_view()[0]!=65||const_view()[1]||static_view()[0]!=65||static_view()[1]||high_view()[0]!=255||high_view()[1]||lengths()!=8||a[0]!=65||a[1]||b[0]!=65||b[1]||c[0]!=65||c[1]||d[0]!=65||d[1]||c_view()!=d)return 1;printf("storage %zu %u %u\n",lengths(),c[0],c[1]);return 0;}
