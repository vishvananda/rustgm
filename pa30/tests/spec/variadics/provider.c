#include <stdarg.h>
#include <stdint.h>
_Static_assert(sizeof(unsigned long)==8,"x86-64 LP64 host");
int64_t sum(uint32_t n,...){va_list a;va_start(a,n);int64_t total=0;for(uint32_t i=0;i<n;i++)total+=va_arg(a,int64_t);va_end(a);return total;}
double sum_float(uint32_t n,...){va_list a;va_start(a,n);double total=0;for(uint32_t i=0;i<n;i++)total+=va_arg(a,double);va_end(a);return total;}
uint32_t ignores(uint32_t n,...){return n;}
uint32_t promoted(uint32_t tag,...){va_list a;va_start(a,tag);int x=va_arg(a,int);int y=va_arg(a,int);int z=va_arg(a,int);double f=va_arg(a,double);va_end(a);return x==7&&y==-3&&z==1&&f==1.5;}
int64_t callback(int64_t(*f)(uint32_t,...)){return f(3,(int64_t)4,(int64_t)5,(int64_t)6);}
uint32_t mixed(uint32_t tag,...){va_list a;va_start(a,tag);int x=va_arg(a,int);unsigned y=va_arg(a,unsigned);int64_t z=va_arg(a,int64_t);uint64_t w=va_arg(a,uint64_t);double f=va_arg(a,double);const uint8_t*p=va_arg(a,const uint8_t*);va_end(a);return x==-7&&y==11&&z==-13&&w==17&&f==2.5&&*p==19;}
typedef struct{uint64_t a;double b;}Pair;
uint32_t record_tail(uint32_t tag,...){va_list a;va_start(a,tag);Pair p=va_arg(a,Pair);va_end(a);return p.a==7&&p.b==2.5;}
Pair record_result(Pair p,...){va_list a;va_start(a,p);p.a+=va_arg(a,uint64_t);p.b+=va_arg(a,double);va_end(a);return p;}
typedef struct{uint32_t a;float b;}Inner;typedef struct{uint32_t head;Inner a[1];float tail;}Value;
uint32_t nested_tail(uint32_t tag,...){va_list a;va_start(a,tag);Value p=va_arg(a,Value);va_end(a);return p.head==7&&p.a[0].a==11&&p.a[0].b==1.5&&p.tail==2.5;}
