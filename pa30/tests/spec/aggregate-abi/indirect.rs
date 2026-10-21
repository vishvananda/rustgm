#[derive(Copy,Clone)] #[repr(C)] struct Mixed{a:f64,b:i64}
#[derive(Copy,Clone)] #[repr(C)] struct Dispatch{f:extern "C" fn(Mixed)->Mixed,tag:u64}
#[derive(Copy,Clone)] #[repr(C)] struct Borrowed{p:&'static u32,tag:u64}
#[unsafe(no_mangle)] pub extern "C" fn export_borrowed(x:Borrowed)->Borrowed{x}
unsafe extern "C"{safe fn host_dispatch()->Dispatch;safe fn host_borrowed()->Borrowed;safe fn check_borrowed()->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let d=host_dispatch();check(d.tag==7);let r=(d.f)(Mixed{a:1.5,b:77});check(r.a==2.5&&r.b==78);let b=host_borrowed();check(*b.p==9&&b.tag==8);check(check_borrowed()==1);}
