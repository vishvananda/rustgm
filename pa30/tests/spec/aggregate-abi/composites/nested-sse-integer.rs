fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]struct Inner{a:f64}#[repr(C)]struct Value{inner:Inner,tail:u64}
#[unsafe(no_mangle)]pub extern "C" fn export_echo(x:Value)->Value{x}
#[unsafe(no_mangle)]pub extern "C" fn export_integer(a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,x:Value,tail:f64)->Value{check(a+b+c+d+e+f==21&&tail==9.5);x}
#[unsafe(no_mangle)]pub extern "C" fn export_sse(a:f64,b:f64,c:f64,d:f64,e:f64,f:f64,g:f64,x:Value,tail:f64)->Value{check(a+b+c+d+e+f+g+tail==36.0);x}
unsafe extern "C"{safe fn host_echo(x:Value)->Value;safe fn host_callback(f:extern "C" fn(Value)->Value,x:Value)->Value;safe fn host_check()->u32;safe fn host_integer(a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,x:Value,tail:f64)->Value;safe fn host_sse(a:f64,b:f64,c:f64,d:f64,e:f64,f:f64,g:f64,x:Value,tail:f64)->Value;}
fn main(){let r=host_echo(Value{inner:Inner{a:1.5},tail:33});check(r.inner.a==1.5&&r.tail==33);let r=host_callback(export_echo,Value{inner:Inner{a:1.5},tail:33});check(r.inner.a==1.5&&r.tail==33);let r=host_integer(1,2,3,4,5,6,Value{inner:Inner{a:1.5},tail:33},9.5);check(r.inner.a==1.5&&r.tail==33);let r=host_sse(1.0,2.0,3.0,4.0,5.0,6.0,7.0,Value{inner:Inner{a:1.5},tail:33},8.0);check(r.inner.a==1.5&&r.tail==33);check(host_check()==1);}
