#include <stdint.h>
typedef struct {double a;} Inner;typedef struct {Inner inner;unsigned long tail;} Value;
_Static_assert(sizeof(Value)==16,"size");_Static_assert(_Alignof(Value)==8,"alignment");
extern Value export_echo(Value);extern Value export_integer(unsigned long,unsigned long,unsigned long,unsigned long,unsigned long,unsigned long,Value,double);extern Value export_sse(double,double,double,double,double,double,double,Value,double);
Value host_echo(Value x){return x;}Value host_callback(Value(*f)(Value),Value x){return f(x);}
Value host_integer(unsigned long a,unsigned long b,unsigned long c,unsigned long d,unsigned long e,unsigned long f,Value x,double tail){if(a+b+c+d+e+f!=21||tail!=9.5)__builtin_trap();return x;}
Value host_sse(double a,double b,double c,double d,double e,double f,double g,Value x,double tail){if(a+b+c+d+e+f+g+tail!=36.0)__builtin_trap();return x;}
unsigned host_check(void){{Value r=export_echo((Value){{1.5},33});if(!(r.inner.a==1.5&&r.tail==33))return 0;}{Value r=export_integer(1,2,3,4,5,6,(Value){{1.5},33},9.5);if(!(r.inner.a==1.5&&r.tail==33))return 0;}{Value r=export_sse(1,2,3,4,5,6,7,(Value){{1.5},33},8);if(!(r.inner.a==1.5&&r.tail==33))return 0;}return 1;}
