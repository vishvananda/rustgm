#include <stdint.h>
#include <stdio.h>
extern uint8_t relay(const uint8_t **);
extern uint8_t ref_relay(const uint8_t ***);
extern uint8_t mixed_relay(const uint8_t *);
int main(void){uint8_t n=13;const uint8_t *p=&n;const uint8_t **q=&p;printf("%u %u %u ",relay(&p),ref_relay(&q),mixed_relay(&n));n=29;printf("%u %u %u\n",relay(&p),ref_relay(&q),mixed_relay(&n));}
