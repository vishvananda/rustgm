static mut DROPS:u32=0;static mut SUM:u32=0;
#[repr(C)]struct D(u8);impl Drop for D{fn drop(&mut self){unsafe{DROPS+=1;SUM+=self.0 as u32;}}}
const fn make()->impl Sized{D(7)}
fn main(){{let _x=const{let mut x=core::mem::ManuallyDrop::new(make());unsafe{*(&raw mut x as *mut u8)=11;core::ptr::read(&raw const x as *const D)}};}unsafe{if DROPS!=1||SUM!=11{let _=fail(0);}}}fn fail(x:u8)->u8{1/x}
