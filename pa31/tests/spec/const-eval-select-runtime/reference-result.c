#include <stdio.h>
extern const unsigned char*answer(const unsigned char*);int main(void){unsigned char value=37;const unsigned char*result=answer(&value);printf("%u %u\n",*result,result==&value);return 0;}
