#include <stdint.h>
struct Mixed{uint64_t n;double x;};
struct Nested{uint16_t cells[2][2];uint64_t tag;};
struct Large{uint64_t cells[5];};
extern struct Mixed metadata_mixed(struct Mixed);
extern struct Nested metadata_nested(struct Nested);
extern struct Large metadata_large(struct Large);
int main(void){struct Mixed m=metadata_mixed((struct Mixed){7,1.5});struct Nested n=metadata_nested((struct Nested){{{1,2},{3,4}},5});struct Large a=metadata_large((struct Large){{1,2,3,4,5}});return !(m.n==10&&m.x==4.0&&n.cells[1][1]==5&&n.tag==7&&a.cells[0]==1&&a.cells[4]==12);}
