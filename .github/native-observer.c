#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
extern uint8_t probe(bool);
extern uint8_t lazy(void);
extern uint8_t user_abort(void) __asm__("abort");
void marker(void){puts("interposed");fflush(stdout);}
void dropped(void){puts("dropped");fflush(stdout);}
int main(int argc,char **argv){
 (void)argv;
 if(argc>1){puts("before");fflush(stdout);uint8_t result=probe(true);printf("after %u\n",(unsigned)result);return 93;}
 uint8_t first=user_abort();uint8_t second=probe(false);uint8_t third=lazy();
 printf("%u %u %u\n",(unsigned)first,(unsigned)second,(unsigned)third);return 0;
}
