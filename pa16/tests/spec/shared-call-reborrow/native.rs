unsafe extern "C" {fn seen(slot:u8,value:u8);}
fn read(r:&u8)->u8{*r}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{
 let x=9u8;let refs=(&x,);let f:fn(&u8)->u8=read;
 unsafe{seen(1,read(refs.0));seen(2,f(refs.0));}
 let mut y=13u8;let r=&mut y;
 unsafe{seen(3,read(r));seen(4,read(r));}
 7
}
