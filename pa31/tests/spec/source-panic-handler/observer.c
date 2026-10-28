#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
extern uint32_t run(bool);
bool evaluated(bool value){fprintf(stderr,"E%u;",value);return value;}
void dropped(void){fputs("D;",stderr);}
void seen_handler(uint32_t line,uint32_t column,bool can_unwind,bool backtrace){fprintf(stderr,"H%u:%u:%u:%u;",line,column,can_unwind,backtrace);}
int main(int argc,char **argv){uint32_t result=run(argc==1||argv[1][0]=='1');printf("returned%u\n",result);return result==7?0:1;}
