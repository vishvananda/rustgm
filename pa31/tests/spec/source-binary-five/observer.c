#include <stdio.h>
int observe_mul(void);int default_mul(void);
int observe_rem(void);int default_rem(void);
int observe_bitxor(void);int default_bitxor(void);
int observe_bitand(void);int default_bitand(void);
int observe_shr(void);int default_shr(void);
int main(void){printf("%d %d %d %d %d %d %d %d %d %d\n",observe_mul(),observe_rem(),observe_bitxor(),observe_bitand(),observe_shr(),default_mul(),default_rem(),default_bitxor(),default_bitand(),default_shr());return 0;}
