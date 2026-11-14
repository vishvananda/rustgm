#include <stdio.h>
#include <stdint.h>
struct Row{const uint8_t *data;uintptr_t len;};struct Table{struct Row rows[2];};extern const struct Table *table(void);int main(void){const struct Table *p=table();printf("%u %u %u %u %llu %llu\n",p->rows[0].data[0],p->rows[0].data[1],p->rows[1].data[0],p->rows[1].data[1],(unsigned long long)p->rows[0].len,(unsigned long long)p->rows[1].len);}
