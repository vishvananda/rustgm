#include <stdio.h>
extern unsigned char none8(void);
extern unsigned char some8(void);
extern unsigned none32(void);
extern unsigned some32(void);
extern unsigned short bytes(void);
int main(void){
  printf("none8 %u\nsome8 %u\n",(unsigned)none8(),(unsigned)some8());
  printf("none32 %u\nsome32 %u\n",none32(),some32());
  printf("bytes %u\n",(unsigned)bytes());
  return 0;
}
