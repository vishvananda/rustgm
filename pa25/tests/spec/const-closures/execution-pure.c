/* Authored observer: runtime operand data crosses the Rust object boundary. */
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
extern size_t pure_shared_constant(void);
extern size_t pure_mutable_constant(void);
extern size_t pure_once_constant(void);
extern size_t pure_runtime_shared(size_t);
extern size_t pure_runtime_mutable(size_t,size_t);
int main(int argc,char **argv){
 if(argc!=2)return 2;
 size_t value=(size_t)strtoull(argv[1],0,10);
 printf("closure-source %zu %zu %zu %zu %zu\n",pure_shared_constant(),pure_mutable_constant(),pure_once_constant(),pure_runtime_shared(value),pure_runtime_mutable(value,47));
 return 0;
}
