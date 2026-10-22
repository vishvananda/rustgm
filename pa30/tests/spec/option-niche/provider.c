#include <stdint.h>
const uint32_t *ptr_echo(const uint32_t *p){return p;}
typedef uint32_t (*Func)(uint32_t);
Func fn_echo(Func f){return f;}
typedef struct{const uint32_t *a;uint64_t b;}Record;
Record record_echo(Record r){r.b+=1;return r;}
