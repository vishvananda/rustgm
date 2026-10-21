typedef struct {_Alignas(16) double a;} Aligned;
extern Aligned export_aligned(Aligned);
Aligned host_aligned(Aligned x){return x;}
unsigned int check_aligned(void){Aligned r=export_aligned((Aligned){3.5});return r.a==3.5 && sizeof(Aligned)==16 && _Alignof(Aligned)==16;}
