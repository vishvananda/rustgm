#include <stddef.h>
#include <stdio.h>
extern const signed char *byte_word_raw(void);
extern const unsigned char *byte_word_view(void);
extern size_t byte_word_length(void);
extern unsigned char byte_word_plain(void),byte_word_generic(void);
int main(void){const unsigned char*a=(const unsigned char*)byte_word_raw(),*b=byte_word_view();unsigned p=byte_word_plain(),g=byte_word_generic();if(byte_word_length()!=2||a[0]!=255||a[1]||b[0]!=255||b[1]||a!=b||p!=17||g!=23)return 1;printf("byte-word %u %u %u %u\n",a[0],b[1],p,g);return 0;}
