#include <stdio.h>
typedef float F __attribute__((vector_size(16)));
typedef double D __attribute__((vector_size(16)));
typedef long long I __attribute__((vector_size(16)));
extern F echo_f(F);extern D echo_d(D);extern I echo_i(I);extern F rust_bridge(F);
int main(void){F f={1.25f,-2.f,3.5f,4.f};D d={5.5,-6.25};I i={7,-8};F a=echo_f(f),b=rust_bridge(f);D c=echo_d(d);I e=echo_i(i);printf("%.2f %.2f %.2f %.2f %.2f %.2f %lld %lld %.2f\n",(double)a[0],(double)a[1],(double)a[2],(double)a[3],c[0],c[1],e[0],e[1],(double)b[2]);return 0;}
