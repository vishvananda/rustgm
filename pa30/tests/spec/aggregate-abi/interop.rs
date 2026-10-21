fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Copy,Clone)] #[repr(C)] struct Byte{a:u8}
#[unsafe(no_mangle)] pub extern "C" fn export_byte(x:Byte)->Byte{x}
#[derive(Copy,Clone)] #[repr(C)] struct Word{a:u32}
#[unsafe(no_mangle)] pub extern "C" fn export_word(x:Word)->Word{x}
#[derive(Copy,Clone)] #[repr(C)] struct Boolean{a:bool}
#[unsafe(no_mangle)] pub extern "C" fn export_boolean(x:Boolean)->Boolean{x}
#[derive(Copy,Clone)] #[repr(C)] struct Float{a:f32}
#[unsafe(no_mangle)] pub extern "C" fn export_float(x:Float)->Float{x}
#[derive(Copy,Clone)] #[repr(C)] struct Floats{a:f32,b:f32}
#[unsafe(no_mangle)] pub extern "C" fn export_floats(x:Floats)->Floats{x}
#[derive(Copy,Clone)] #[repr(C)] struct Doubles{a:f64,b:f64}
#[unsafe(no_mangle)] pub extern "C" fn export_doubles(x:Doubles)->Doubles{x}
#[derive(Copy,Clone)] #[repr(C)] struct Mixed{a:f64,b:i64}
#[unsafe(no_mangle)] pub extern "C" fn export_mixed(x:Mixed)->Mixed{x}
#[derive(Copy,Clone)] #[repr(C)] struct Inverse{a:i64,b:f64}
#[unsafe(no_mangle)] pub extern "C" fn export_inverse(x:Inverse)->Inverse{x}
#[derive(Copy,Clone)] #[repr(C)] struct Pair{a:i64,b:i64}
#[unsafe(no_mangle)] pub extern "C" fn export_pair(x:Pair)->Pair{x}
#[derive(Copy,Clone)] #[repr(C)] struct Large{a:i64,b:i64,c:i64}
#[unsafe(no_mangle)] pub extern "C" fn export_large(x:Large)->Large{x}
#[derive(Copy,Clone)] #[repr(C)] struct Padded{a:u8,b:u64}
#[unsafe(no_mangle)] pub extern "C" fn export_padded(x:Padded)->Padded{x}
#[derive(Copy,Clone)] #[repr(C)] struct Wide{a:u128}
#[unsafe(no_mangle)] pub extern "C" fn export_wide(x:Wide)->Wide{x}
#[derive(Copy,Clone)] #[repr(C)] struct Pointer{a:*const u8,b:u64}
#[unsafe(no_mangle)] pub extern "C" fn export_pointer(x:Pointer)->Pointer{x}
#[repr(C)] struct Droppy(u32);impl Drop for Droppy{fn drop(&mut self){host_mark_drop();}}
#[unsafe(no_mangle)] pub extern "C" fn export_drop(x:Droppy)->u32{x.0}
#[unsafe(no_mangle)] pub extern "C" fn integer_rollback(a:i64,b:i64,c:i64,d:i64,e:i64,p:Pair,tail:i64)->Pair{check(a+b+c+d+e+tail==264);p}
#[unsafe(no_mangle)] pub extern "C" fn sse_rollback(a:f64,b:f64,c:f64,d:f64,e:f64,f:f64,g:f64,p:Doubles,tail:f64)->Doubles{check(a+b+c+d+e+f+g+tail==37.5);p}
unsafe extern "C"{safe fn host_byte(x:Byte)->Byte;safe fn host_word(x:Word)->Word;safe fn host_boolean(x:Boolean)->Boolean;safe fn host_float(x:Float)->Float;safe fn host_floats(x:Floats)->Floats;safe fn host_doubles(x:Doubles)->Doubles;safe fn host_mixed(x:Mixed)->Mixed;safe fn host_inverse(x:Inverse)->Inverse;safe fn host_pair(x:Pair)->Pair;safe fn host_large(x:Large)->Large;safe fn host_padded(x:Padded)->Padded;safe fn host_wide(x:Wide)->Wide;safe fn host_pointer(x:Pointer)->Pointer;safe fn host_mark_drop();safe fn host_check()->u32;safe fn host_callback(f:extern "C" fn(Mixed)->Mixed,x:Mixed)->Mixed;safe fn host_integer_rollback(a:i64,b:i64,c:i64,d:i64,e:i64,p:Pair,tail:i64)->Pair;safe fn host_sse_rollback(a:f64,b:f64,c:f64,d:f64,e:f64,f:f64,g:f64,p:Doubles,tail:f64)->Doubles;}
fn main(){
{let r=host_pointer(Pointer{a:0 as *const u8,b:77});check(r.a==0 as *const u8&&r.b==77);}
{let r=host_byte(Byte{a:7});check(r.a==7);}
{let r=host_word(Word{a:77});check(r.a==77);}
{let r=host_boolean(Boolean{a:true});check(r.a==true);}
{let r=host_float(Float{a:1.5});check(r.a==1.5);}
{let r=host_floats(Floats{a:1.5,b:2.5});check(r.a==1.5&&r.b==2.5);}
{let r=host_doubles(Doubles{a:1.5,b:2.5});check(r.a==1.5&&r.b==2.5);}
{let r=host_mixed(Mixed{a:1.5,b:77});check(r.a==1.5&&r.b==77);}
{let r=host_inverse(Inverse{a:77,b:2.5});check(r.a==77&&r.b==2.5);}
{let r=host_pair(Pair{a:22,b:33});check(r.a==22&&r.b==33);}
{let r=host_large(Large{a:11,b:22,c:33});check(r.a==11&&r.b==22&&r.c==33);}
{let r=host_padded(Padded{a:7,b:77});check(r.a==7&&r.b==77);}
{let r=host_wide(Wide{a:18446744073709551623});check(r.a==18446744073709551623);}
let p=host_integer_rollback(11,22,33,44,55,Pair{a:200,b:300},99);check(p.a==200&&p.b==300);
let s=host_sse_rollback(1.0,2.0,3.0,4.0,5.0,6.0,7.0,Doubles{a:20.5,b:30.5},9.5);check(s.a==20.5&&s.b==30.5);
let m=host_callback(export_mixed,Mixed{a:1.5,b:77});check(m.a==1.5&&m.b==77);check(host_check()==1);}
