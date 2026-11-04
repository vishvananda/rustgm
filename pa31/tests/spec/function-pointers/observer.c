#include <stdio.h>
typedef unsigned char (*callback)(void);
extern const void *direct(callback);extern const void *bounded(callback);extern const void *unsafe_pointer(callback);extern const void *reified(callback);
extern const void *once(void);extern const void *hrtb_address(void);extern const void *scoped_address(void);
extern const unsigned char *forwarded_borrow(const unsigned char *);
static unsigned calls;static unsigned makers;
static unsigned char effect(void){++calls;return 17;}
callback make_callback(void){++makers;return effect;}
int main(void){
 const void *a=direct(effect);const void *b=bounded(effect);const void *c=unsafe_pointer(effect);const void *d=reified(effect);
 const void *e=once();const void *f=hrtb_address();const void *g=scoped_address();
 printf("%u\n%u\n%u\n%u\n%u\n%u\n%u\n%u\n%u\n",a==(const void*)effect,b==(const void*)effect,c==(const void*)effect,d==(const void*)effect,e==(const void*)effect,f==(const void*)forwarded_borrow,g==(const void*)forwarded_borrow,calls,makers);return 0;
}
