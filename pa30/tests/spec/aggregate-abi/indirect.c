typedef struct {double a;long b;} Mixed;
typedef Mixed (*Callback)(Mixed);
typedef struct {Callback f;unsigned long tag;} Dispatch;
typedef struct {const unsigned int *p;unsigned long tag;} Borrowed;
static const unsigned int value=9;
static Mixed add(Mixed x){return (Mixed){x.a+1.0,x.b+1};}
Dispatch host_dispatch(void){return (Dispatch){add,7};}
Borrowed host_borrowed(void){return (Borrowed){&value,8};}
extern Borrowed export_borrowed(Borrowed);
unsigned int check_borrowed(void){Borrowed r=export_borrowed((Borrowed){&value,8});return r.p==&value && *r.p==9 && r.tag==8;}
