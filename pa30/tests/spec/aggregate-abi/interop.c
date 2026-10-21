#include <stdbool.h>
typedef struct {unsigned char a;} Byte;
extern Byte export_byte(Byte);
Byte host_byte(Byte x){return x;}
typedef struct {unsigned int a;} Word;
extern Word export_word(Word);
Word host_word(Word x){return x;}
typedef struct {_Bool a;} Boolean;
extern Boolean export_boolean(Boolean);
Boolean host_boolean(Boolean x){return x;}
typedef struct {float a;} Float;
extern Float export_float(Float);
Float host_float(Float x){return x;}
typedef struct {float a; float b;} Floats;
extern Floats export_floats(Floats);
Floats host_floats(Floats x){return x;}
typedef struct {double a; double b;} Doubles;
extern Doubles export_doubles(Doubles);
Doubles host_doubles(Doubles x){return x;}
typedef struct {double a; long b;} Mixed;
extern Mixed export_mixed(Mixed);
Mixed host_mixed(Mixed x){return x;}
typedef struct {long a; double b;} Inverse;
extern Inverse export_inverse(Inverse);
Inverse host_inverse(Inverse x){return x;}
typedef struct {long a; long b;} Pair;
extern Pair export_pair(Pair);
Pair host_pair(Pair x){return x;}
typedef struct {long a; long b; long c;} Large;
extern Large export_large(Large);
Large host_large(Large x){return x;}
typedef struct {unsigned char a; unsigned long b;} Padded;
extern Padded export_padded(Padded);
Padded host_padded(Padded x){return x;}
typedef struct {unsigned __int128 a;} Wide;
extern Wide export_wide(Wide);
Wide host_wide(Wide x){return x;}
typedef struct {const unsigned char *a;unsigned long b;} Pointer;extern Pointer export_pointer(Pointer);Pointer host_pointer(Pointer x){return x;}
typedef struct {unsigned int a;} Droppy;extern unsigned int export_drop(Droppy);static unsigned int drops;void host_mark_drop(void){drops++;}
Mixed host_callback(Mixed (*f)(Mixed),Mixed x){return f(x);}
extern Pair integer_rollback(long,long,long,long,long,Pair,long);Pair host_integer_rollback(long a,long b,long c,long d,long e,Pair p,long tail){if(a+b+c+d+e+tail!=264)p.a=0;return p;}
extern Doubles sse_rollback(double,double,double,double,double,double,double,Doubles,double);Doubles host_sse_rollback(double a,double b,double c,double d,double e,double f,double g,Doubles p,double tail){if(a+b+c+d+e+f+g+tail!=37.5)p.a=0;return p;}
unsigned int host_check(void){
{Byte r=export_byte((Byte){7});if(r.a!=7)return 0;}
{Word r=export_word((Word){77});if(r.a!=77)return 0;}
{Boolean r=export_boolean((Boolean){1});if(r.a!=1)return 0;}
{Float r=export_float((Float){1.5});if(r.a!=1.5)return 0;}
{Floats r=export_floats((Floats){1.5,2.5});if(r.a!=1.5||r.b!=2.5)return 0;}
{Doubles r=export_doubles((Doubles){1.5,2.5});if(r.a!=1.5||r.b!=2.5)return 0;}
{Mixed r=export_mixed((Mixed){1.5,77});if(r.a!=1.5||r.b!=77)return 0;}
{Inverse r=export_inverse((Inverse){77,2.5});if(r.a!=77||r.b!=2.5)return 0;}
{Pair r=export_pair((Pair){22,33});if(r.a!=22||r.b!=33)return 0;}
{Large r=export_large((Large){11,22,33});if(r.a!=11||r.b!=22||r.c!=33)return 0;}
{Padded r=export_padded((Padded){7,77});if(r.a!=7||r.b!=77)return 0;}
{Wide r=export_wide((Wide){(((unsigned __int128)1<<64)+7)});if(r.a!=(((unsigned __int128)1<<64)+7))return 0;}
{Pair p={200,300};Pair r=integer_rollback(11,22,33,44,55,p,99);if(r.a!=200||r.b!=300)return 0;}
{Doubles p={20.5,30.5};Doubles r=sse_rollback(1,2,3,4,5,6,7,p,9.5);if(r.a!=20.5||r.b!=30.5)return 0;}
{Pointer r=export_pointer((Pointer){0,77});if(r.a!=0||r.b!=77)return 0;}
if(export_drop((Droppy){7})!=7||drops!=1){return 0;}
return 1;}
