fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Code{fn code(self)->u32;}
impl<'a> Code for &'a u8{fn code(self)->u32{*self as u32}}
impl Code for u8{fn code(self)->u32{self as u32}}
fn wrap(site:u32,m:impl Code)->u32{site+m.code()}
fn call<F:FnOnce()->u32>(f:F)->u32{f()}
fn main(){let site=10;let v=3u8;let error=|message|wrap(site,message);check(call(||error(&v))==13);check(error(3u8)==13);}
