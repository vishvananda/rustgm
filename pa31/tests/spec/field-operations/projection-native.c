#include <stdint.h>
#include <stdio.h>
struct Record { uint8_t byte; uint32_t word; };
extern uint32_t projected_word(uint32_t);
extern const struct Record *projected_base(const struct Record *);
extern uint32_t projected_callback(void);
static int calls;
uint32_t next_word(void) { ++calls; return 29; }
int main(void) {
    struct Record record = { 3, 47 };
    uint32_t word = projected_word(23);
    uint32_t callback = projected_callback();
    int same = projected_base(&record) == &record;
    printf("projection %u %u %d %d\n", (unsigned)word, (unsigned)callback, same, calls);
    return word != 23 || callback != 29 || !same || calls != 1;
}
