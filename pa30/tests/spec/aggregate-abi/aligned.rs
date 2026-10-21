#[derive(Copy,Clone)] #[repr(C,align(16))] struct Aligned{a:f64}
#[unsafe(no_mangle)] pub extern "C" fn export_aligned(x:Aligned)->Aligned{x}
unsafe extern "C"{safe fn host_aligned(x:Aligned)->Aligned;safe fn check_aligned()->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let r=host_aligned(Aligned{a:3.5});check(r.a==3.5);check(check_aligned()==1);}
