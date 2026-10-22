#include <stdarg.h>
#include <stdint.h>
typedef struct{uint64_t a;double b;}Pair;
uint32_t pairs(uint32_t tag,...){va_list args;va_start(args,tag);Pair x=va_arg(args,Pair);int marker=va_arg(args,int);Pair y=va_arg(args,Pair);va_end(args);return tag==3&&x.a==7&&x.b==1.5&&marker==-9&&y.a==11&&y.b==2.5;}
uint32_t ignores_tail(uint32_t tag,...){return tag;}
int64_t sum_tail(uint32_t n,...){va_list args;va_start(args,n);int64_t value=0;for(uint32_t i=0;i<n;i++)value+=va_arg(args,int64_t);va_end(args);return value;}
